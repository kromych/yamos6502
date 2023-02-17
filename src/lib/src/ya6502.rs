//! Behavioral emulator of MOS 6502
//!
//! There is no cycle-accurate emulation, support
//! for undocumented instructions and other microarch
//! side effects such as writing the old value first
//! for the read-modify-write instructions.

use core::fmt::Debug;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::insns::Insn;
use crate::AddressMode;
use bitflags::bitflags;

/// Memory errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryError {
    /// Could not read at the address
    BadAddress(u16),
    /// Could not write at the address
    ReadOnlyAddress(u16),
}

/// A 16-bit addressable, 8-bit cell memory
pub trait Memory {
    fn write(&mut self, addr: u16, value: u8) -> Result<(), MemoryError>;
    fn read(&self, addr: u16) -> Result<u8, MemoryError>;
}

bitflags! {
    /// SR Flags (bit 7 to bit 0)
    pub struct Status : u8 {
        /// N	Negative
        const NEG = 0x80;
        /// V	Overflow
        const OVF = 0x40;
        /// -   Ignored (in the register, hardwired to the logic `1`)
        const _IGNORED = 0x20;
        /// B	Break (is never set in the register,
        ///            only in the register value pushed on the stack which
        ///            happens when executing BRK)
        const BRK = 0x10;
        /// D	Decimal (use BCD for arithmetics), cleared on reset
        const BCD = 0x08;
        /// I	Interrupt (IRQ) disable, set on reset
        const INT_DIS = 0x04;
        /// Z	Zero
        const ZERO = 0x02;
        /// C	Carry
        const CARRY = 0x01;
    }
}

/// When an interrupt is signaled, the low and the high
/// 8 bits of the program counter are loaded
/// from these addresses.
pub const IRQ_VECTOR: [u16; 2] = [0xFFFE, 0xFFFF];

/// When a reset is requested, the low and the high
/// 8 bits of the program counter are loaded
/// from these addresses.
pub const RESET_VECTOR: [u16; 2] = [0xFFFC, 0xFFFD];

/// When an non-maskable interrupt is signaled,
/// the low and the high 8 bits of the program counter
/// are loaded from these addresses.
pub const NMI_VECTOR: [u16; 2] = [0xFFFA, 0xFFFB];

/// MOS 6502 register state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mos6502RegisterFile {
    /// Program counter, little-endian
    pub pc: u16,
    /// Stack pointer. The stack grows top-down
    pub sp: u8,
    /// Accumulator
    pub a: u8,
    /// X index register
    pub x: u8,
    /// Y index register
    pub y: u8,
    /// Status register [NV-BDIZC]
    pub p: Status,
}

impl Mos6502RegisterFile {
    /// Some arbitrary values giving an
    /// incosistent state to catch bugs
    pub fn new() -> Self {
        Self {
            pc: 0xFF55,
            a: 0xAA,
            x: 0xCC,
            y: 0xD2,
            sp: 0x01,
            p: Status::empty(),
        }
    }

    pub fn reset(&mut self) {
        // Hardware sets few flags, everything else is initialized
        // by software.
        self.p.set(Status::INT_DIS, true);
        self.p.set(Status::_IGNORED, true);
        self.p.set(Status::BCD, false);

        // Stack pointer is not set! In some configurations that might
        // not even be useful, e.g. if the only type of memory is ROM.
        // The software is expected to initialize the stack pointer to
        // use interrupts and subroutine calls.
    }
}

impl Default for Mos6502RegisterFile {
    fn default() -> Self {
        Mos6502RegisterFile::new()
    }
}

/// Run normal exit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunExit {
    /// Instruction retired
    InstructionExecuted(Insn),
    /// Break instruction was fetched from the address
    Break,
    /// Interrupt
    Interrupt,
    /// Non-maskable interrupt
    NonMaskableInterrupt,
}

/// Run error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunError {
    /// Malformed or unknown instruction was fetched
    InvalidInstruction(u8),
    /// Could not fetch the next instruction from the address
    CannotFetchInstruction(MemoryError),
    /// Stack overflow
    StackOverflow,
    /// Error occured when accessing the memory
    MemoryAccess(MemoryError),
}

/// Behavioral MOS 6502 emulator
#[derive(Debug)]
pub struct Mos6502<'memory, M>
where
    M: Memory,
{
    mem: &'memory mut M,
    regf: Mos6502RegisterFile,
    // The target might not provide better options than
    // plain atomic store and loads. Should be a room
    // for perf optimization.
    reset_pending: AtomicBool,
    irq_pending: AtomicBool,
    nmi_pending: AtomicBool,
    // Jammed, only reset will help
    fault: Option<RunError>,
    last_opcode: u8,
}

impl<'memory, M> Mos6502<'memory, M>
where
    M: Memory,
{
    pub fn new(memory: &'memory mut M) -> Self {
        Self {
            mem: memory,
            regf: Mos6502RegisterFile::default(),
            reset_pending: AtomicBool::new(false),
            nmi_pending: AtomicBool::new(false),
            irq_pending: AtomicBool::new(false),
            fault: None,
            last_opcode: 0,
        }
    }

    pub fn with_registers(memory: &'memory mut M, regf: Mos6502RegisterFile) -> Self {
        Self {
            mem: memory,
            regf,
            reset_pending: AtomicBool::new(false),
            nmi_pending: AtomicBool::new(false),
            irq_pending: AtomicBool::new(false),
            fault: None,
            last_opcode: 0,
        }
    }

    pub fn irq(&mut self) {
        self.irq_pending.store(true, Ordering::Release);
    }

    pub fn reset(&mut self) {
        self.reset_pending.store(true, Ordering::Release);
    }

    pub fn nmi(&mut self) {
        self.nmi_pending.store(true, Ordering::Release);
    }

    pub fn registers(&self) -> &Mos6502RegisterFile {
        &self.regf
    }

    fn load_u8(&self, addr: u16) -> Result<u8, RunError> {
        self.mem.read(addr).map_err(RunError::MemoryAccess)
    }

    fn store_u8(&mut self, addr: u16, value: u8) -> Result<(), RunError> {
        self.mem.write(addr, value).map_err(RunError::MemoryAccess)
    }

    fn get_u16_at(&self, addr: u16) -> Result<u16, RunError> {
        let lo = self.mem.read(addr).map_err(RunError::MemoryAccess)?;
        let hi = self
            .mem
            .read(addr.wrapping_add(1))
            .map_err(RunError::MemoryAccess)?;

        Ok(u16::from_le_bytes([lo, hi]))
    }

    /// Computes the effective address. Expects the program counter being advanced past
    /// the instruction opcode, and advances it to skip the addressing mode bytes.
    fn get_effective_address(&mut self, addr_mode: AddressMode) -> Result<u16, RunError> {
        match addr_mode {
            AddressMode::Implicit => Err(RunError::InvalidInstruction(self.last_opcode)),
            AddressMode::Immediate | AddressMode::Relative => {
                let ea = self.regf.pc;
                self.regf.pc = self.regf.pc.wrapping_add(1);

                Ok(ea)
            }
            AddressMode::Indirect => {
                let ptr = self.get_u16_at(self.regf.pc)?;
                self.regf.pc = self.regf.pc.wrapping_add(2);
                let ea = self.get_u16_at(ptr)?;

                Ok(ea)
            }
            AddressMode::Xindirect => {
                let ptr = self.load_u8(self.regf.pc)?.wrapping_add(self.regf.x).into();
                self.regf.pc = self.regf.pc.wrapping_add(1);
                let ea = self.get_u16_at(ptr)?;

                Ok(ea)
            }
            AddressMode::IndirectY => {
                let ptr = self.load_u8(self.regf.pc)?.into();
                self.regf.pc = self.regf.pc.wrapping_add(1);
                let ea = self.get_u16_at(ptr)?.wrapping_add(self.regf.y.into());

                Ok(ea)
            }
            AddressMode::Absolute => {
                let ea = self.get_u16_at(self.regf.pc)?;
                self.regf.pc = self.regf.pc.wrapping_add(2);

                Ok(ea)
            }
            AddressMode::AbsoluteX => {
                let ea = self
                    .get_u16_at(self.regf.pc)?
                    .wrapping_add(self.regf.x.into());
                self.regf.pc = self.regf.pc.wrapping_add(2);

                Ok(ea)
            }
            AddressMode::AbsoluteY => {
                let ea = self
                    .get_u16_at(self.regf.pc)?
                    .wrapping_add(self.regf.y.into());
                self.regf.pc = self.regf.pc.wrapping_add(2);

                Ok(ea)
            }
            AddressMode::Zeropage => {
                let ea = self.load_u8(self.regf.pc)?.into();
                self.regf.pc = self.regf.pc.wrapping_add(1);

                Ok(ea)
            }
            AddressMode::ZeropageX => {
                let ea = self.load_u8(self.regf.pc)?.wrapping_add(self.regf.x).into();
                self.regf.pc = self.regf.pc.wrapping_add(1);

                Ok(ea)
            }
            AddressMode::ZeropageY => {
                let ea = self.load_u8(self.regf.pc)?.wrapping_add(self.regf.y).into();
                self.regf.pc = self.regf.pc.wrapping_add(1);

                Ok(ea)
            }
        }
    }

    fn jump_indirect(&mut self, pc_ptr: [u16; 2]) -> Result<(), RunError> {
        let lo_pc = self.mem.read(pc_ptr[0]).map_err(RunError::MemoryAccess)?;
        let hi_pc = self.mem.read(pc_ptr[1]).map_err(RunError::MemoryAccess)?;
        self.regf.pc = u16::from_le_bytes([lo_pc, hi_pc]);

        Ok(())
    }

    fn step(&mut self) -> Result<RunExit, RunError> {
        // Fetch instruction
        self.last_opcode = self
            .mem
            .read(self.regf.pc)
            .map_err(RunError::CannotFetchInstruction)?;
        let insn = crate::insns::get_insn_by_opcode(self.last_opcode);
        match insn {
            // Group 0b00. Flags, conditionals, jumps, misc. There are a few
            // quite complex instructions here.
            Insn::BCC(addr_mode) => self.bcc(addr_mode),
            Insn::BCS(addr_mode) => self.bcs(addr_mode),
            Insn::BEQ(addr_mode) => self.beq(addr_mode),
            Insn::BIT(addr_mode) => self.bit(addr_mode),
            Insn::BMI(addr_mode) => self.bmi(addr_mode),
            Insn::BNE(addr_mode) => self.bne(addr_mode),
            Insn::BPL(addr_mode) => self.bpl(addr_mode),
            Insn::BRK => {
                self.brk()?;
                return Ok(RunExit::Break);
            }
            Insn::BVC(addr_mode) => self.bvc(addr_mode),
            Insn::BVS(addr_mode) => self.bvs(addr_mode),
            Insn::CLC => self.clc(),
            Insn::CLD => self.cld(),
            Insn::CLI => self.cli(),
            Insn::CLV => self.clv(),
            Insn::CPX(addr_mode) => self.cpx(addr_mode),
            Insn::CPY(addr_mode) => self.cpy(addr_mode),
            Insn::DEY => self.dey(),
            Insn::INX => self.inx(),
            Insn::INY => self.iny(),
            Insn::JMP(addr_mode) => self.jmp(addr_mode),
            Insn::JSR(addr_mode) => self.jsr(addr_mode),
            Insn::LDY(addr_mode) => self.ldy(addr_mode),
            Insn::PHA => self.pha(),
            Insn::PHP => self.php(),
            Insn::PLA => self.pla(),
            Insn::PLP => self.plp(),
            Insn::RTI => self.rti(),
            Insn::RTS => self.rts(),
            Insn::SEC => self.sec(),
            Insn::SED => self.sed(),
            Insn::SEI => self.sei(),
            Insn::STY(addr_mode) => self.sty(addr_mode),
            Insn::TAY => self.tay(),
            Insn::TYA => self.tya(),
            // Group 0b01. ALU instructions, very regular encoding
            // to make decoding and execution faster in hardware.
            Insn::ADC(addr_mode) => self.adc(addr_mode),
            Insn::AND(addr_mode) => self.and(addr_mode),
            Insn::CMP(addr_mode) => self.cmp(addr_mode),
            Insn::EOR(addr_mode) => self.eor(addr_mode),
            Insn::LDA(addr_mode) => self.lda(addr_mode),
            Insn::ORA(addr_mode) => self.ora(addr_mode),
            Insn::SBC(addr_mode) => self.sbc(addr_mode),
            Insn::STA(addr_mode) => self.sta(addr_mode),
            // Group 0b10. Bit operation and accumulator operations,
            // less regular than the ALU group.
            Insn::ASL(addr_mode) => self.asl(addr_mode),
            Insn::DEC(addr_mode) => self.dec(addr_mode),
            Insn::DEX => self.dex(),
            Insn::INC(addr_mode) => self.inc(addr_mode),
            Insn::LDX(addr_mode) => self.ldx(addr_mode),
            Insn::LSR(addr_mode) => self.lsr(addr_mode),
            Insn::NOP => self.nop(),
            Insn::ROL(addr_mode) => self.rol(addr_mode),
            Insn::ROR(addr_mode) => self.ror(addr_mode),
            Insn::STX(addr_mode) => self.stx(addr_mode),
            Insn::TAX => self.tax(),
            Insn::TSX => self.tsx(),
            Insn::TXA => self.txa(),
            Insn::TXS => self.txs(),
            // Group 0b11 contains invalid instructions
            Insn::JAM => {
                return Err(RunError::InvalidInstruction(self.last_opcode));
            }
        }?;

        Ok(RunExit::InstructionExecuted(insn))
    }

    pub fn run(&mut self) -> Result<RunExit, RunError> {
        // Handle reset.
        // The real processor can't/won't deaasert the line.
        if self.reset_pending.load(Ordering::Acquire) {
            self.fault = None;
            self.regf.reset();
            self.jump_indirect(RESET_VECTOR)?;
            self.reset_pending.store(false, Ordering::Release);
        }

        // If the processor faulted, refuse to run.
        if let Some(f) = self.fault {
            return Err(f);
        }

        // Handle other events.
        // The real processor can't/won't deaasert these lines.
        if self.nmi_pending.load(Ordering::Acquire) {
            self.jump_indirect(NMI_VECTOR)?;
            self.nmi_pending.store(false, Ordering::Release);
            return Ok(RunExit::NonMaskableInterrupt);
        }
        if !self.regf.p.contains(Status::INT_DIS) && self.irq_pending.load(Ordering::Acquire) {
            self.jump_indirect(IRQ_VECTOR)?;
            self.irq_pending.store(false, Ordering::Release);
            return Ok(RunExit::Interrupt);
        }

        // The register state is rolled back on an instruction fault
        let registers = self.regf;
        match self.step() {
            Err(e) => {
                self.fault = Some(e);
                self.regf = registers;
                Err(e)
            }
            Ok(o) => Ok(o),
        }
    }

    #[inline]
    fn bcc(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("bcc")
    }

    #[inline]
    fn bcs(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("bcs")
    }

    #[inline]
    fn beq(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("beq")
    }

    #[inline]
    fn bit(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("bit")
    }

    #[inline]
    fn bmi(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("bmi")
    }

    #[inline]
    fn bne(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("bne")
    }

    #[inline]
    fn bpl(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("bpl")
    }

    #[inline]
    fn brk(&mut self) -> Result<(), RunError> {
        todo!("brk")
    }

    #[inline]
    fn bvc(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("bvc")
    }

    #[inline]
    fn bvs(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("bvs")
    }

    #[inline]
    fn clc(&mut self) -> Result<(), RunError> {
        todo!("clc")
    }

    #[inline]
    fn cld(&mut self) -> Result<(), RunError> {
        todo!("cld")
    }

    #[inline]
    fn cli(&mut self) -> Result<(), RunError> {
        todo!("cli")
    }

    #[inline]
    fn clv(&mut self) -> Result<(), RunError> {
        todo!("clv")
    }

    #[inline]
    fn cpx(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("cpx")
    }

    #[inline]
    fn cpy(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("cpy")
    }

    #[inline]
    fn dey(&mut self) -> Result<(), RunError> {
        todo!("dey")
    }

    #[inline]
    fn inx(&mut self) -> Result<(), RunError> {
        todo!("inx")
    }

    #[inline]
    fn iny(&mut self) -> Result<(), RunError> {
        todo!("iny")
    }

    #[inline]
    fn jmp(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("jmp")
    }

    #[inline]
    fn jsr(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("jsr")
    }

    #[inline]
    fn ldy(&mut self, addr_mode: AddressMode) -> Result<(), RunError> {
        self.regf.pc = self.regf.pc.wrapping_add(1);

        let ea = self.get_effective_address(addr_mode)?;
        let data = self.load_u8(ea)?;
        self.regf.y = data;
        self.regf.p.set(Status::ZERO, self.regf.y == 0);
        self.regf.p.set(Status::NEG, (self.regf.y as i8) < 0);

        Ok(())
    }

    #[inline]
    fn pha(&mut self) -> Result<(), RunError> {
        todo!("pha")
    }

    #[inline]
    fn php(&mut self) -> Result<(), RunError> {
        todo!("php")
    }

    #[inline]
    fn pla(&mut self) -> Result<(), RunError> {
        todo!("pla")
    }

    #[inline]
    fn plp(&mut self) -> Result<(), RunError> {
        todo!("plp")
    }

    #[inline]
    fn rti(&mut self) -> Result<(), RunError> {
        todo!("rti")
    }

    #[inline]
    fn rts(&mut self) -> Result<(), RunError> {
        todo!("rts")
    }

    #[inline]
    fn sec(&mut self) -> Result<(), RunError> {
        todo!("sec")
    }

    #[inline]
    fn sed(&mut self) -> Result<(), RunError> {
        todo!("sed")
    }

    #[inline]
    fn sei(&mut self) -> Result<(), RunError> {
        todo!("sei")
    }

    #[inline]
    fn sty(&mut self, addr_mode: AddressMode) -> Result<(), RunError> {
        self.regf.pc = self.regf.pc.wrapping_add(1);

        let ea = self.get_effective_address(addr_mode)?;
        self.store_u8(ea, self.regf.y)?;

        Ok(())
    }

    #[inline]
    fn tay(&mut self) -> Result<(), RunError> {
        todo!("tay")
    }

    #[inline]
    fn tya(&mut self) -> Result<(), RunError> {
        todo!("tya")
    }

    #[inline]
    fn adc(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("adc")
    }

    #[inline]
    fn and(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("and")
    }

    #[inline]
    fn cmp(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("cmp")
    }

    #[inline]
    fn eor(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("eor")
    }

    #[inline]
    fn lda(&mut self, addr_mode: AddressMode) -> Result<(), RunError> {
        self.regf.pc = self.regf.pc.wrapping_add(1);

        let ea = self.get_effective_address(addr_mode)?;
        let data = self.load_u8(ea)?;
        self.regf.a = data;
        self.regf.p.set(Status::ZERO, self.regf.a == 0);
        self.regf.p.set(Status::NEG, (self.regf.a as i8) < 0);

        Ok(())
    }

    #[inline]
    fn ora(&mut self, addr_mode: AddressMode) -> Result<(), RunError> {
        self.regf.pc = self.regf.pc.wrapping_add(1);

        let ea = self.get_effective_address(addr_mode)?;
        let data = self.load_u8(ea)?;
        self.regf.a |= data;
        self.regf.p.set(Status::ZERO, self.regf.a == 0);
        self.regf.p.set(Status::NEG, (self.regf.a as i8) < 0);

        Ok(())
    }

    #[inline]
    fn sbc(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("sbc")
    }

    #[inline]
    fn sta(&mut self, addr_mode: AddressMode) -> Result<(), RunError> {
        self.regf.pc = self.regf.pc.wrapping_add(1);

        let ea = self.get_effective_address(addr_mode)?;
        self.store_u8(ea, self.regf.a)?;

        Ok(())
    }

    #[inline]
    fn asl(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("asl")
    }

    #[inline]
    fn dec(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("dec")
    }

    #[inline]
    fn dex(&mut self) -> Result<(), RunError> {
        todo!("dex")
    }

    #[inline]
    fn inc(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("inc")
    }

    #[inline]
    fn ldx(&mut self, addr_mode: AddressMode) -> Result<(), RunError> {
        self.regf.pc = self.regf.pc.wrapping_add(1);

        let ea = self.get_effective_address(addr_mode)?;
        let data = self.load_u8(ea)?;
        self.regf.x = data;
        self.regf.p.set(Status::ZERO, self.regf.x == 0);
        self.regf.p.set(Status::NEG, (self.regf.x as i8) < 0);

        Ok(())
    }

    #[inline]
    fn lsr(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("lsr")
    }

    #[inline]
    fn nop(&mut self) -> Result<(), RunError> {
        todo!("nop")
    }

    #[inline]
    fn rol(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("rol")
    }

    #[inline]
    fn ror(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("ror")
    }

    #[inline]
    fn stx(&mut self, addr_mode: AddressMode) -> Result<(), RunError> {
        self.regf.pc = self.regf.pc.wrapping_add(1);

        let ea = self.get_effective_address(addr_mode)?;
        self.store_u8(ea, self.regf.x)?;

        Ok(())
    }

    #[inline]
    fn tax(&mut self) -> Result<(), RunError> {
        todo!("tax")
    }

    #[inline]
    fn tsx(&mut self) -> Result<(), RunError> {
        todo!("tsx")
    }

    #[inline]
    fn txa(&mut self) -> Result<(), RunError> {
        todo!("txa")
    }

    #[inline]
    fn txs(&mut self) -> Result<(), RunError> {
        todo!("txs")
    }
}
