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
use c2rust_bitfields::BitfieldStruct;

#[derive(Copy, Clone, BitfieldStruct, PartialEq, Eq)]
pub struct Opcode {
    #[bitfield(name = "op", ty = "u8", bits = "0..=2")]
    #[bitfield(name = "addr_mode", ty = "u8", bits = "3..=5")]
    #[bitfield(name = "insn_group", ty = "u8", bits = "6..=7")]
    opcode: [u8; 1],
}

impl From<Opcode> for u8 {
    fn from(value: Opcode) -> Self {
        value.opcode[0]
    }
}

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        Opcode { opcode: [value] }
    }
}

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
pub struct Mos6502RegisterState {
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

impl Mos6502RegisterState {
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

impl Default for Mos6502RegisterState {
    fn default() -> Self {
        Mos6502RegisterState::new()
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
    memory: &'memory M,
    registers: Mos6502RegisterState,
    reset_pending: AtomicBool,
    irq_pending: AtomicBool,
    nmi_pending: AtomicBool,
}

impl<'memory, M> Mos6502<'memory, M>
where
    M: Memory,
{
    pub fn new(memory: &'memory M) -> Self {
        Self {
            memory,
            registers: Mos6502RegisterState::default(),
            reset_pending: AtomicBool::new(false),
            nmi_pending: AtomicBool::new(false),
            irq_pending: AtomicBool::new(false),
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

    pub fn registers(&self) -> &Mos6502RegisterState {
        &self.registers
    }

    fn jump_indirect(&mut self, pc_ptr: [u16; 2]) -> Result<(), RunError> {
        let lo_pc = self
            .memory
            .read(pc_ptr[0])
            .map_err(RunError::MemoryAccess)?;
        let hi_pc = self
            .memory
            .read(pc_ptr[1])
            .map_err(RunError::MemoryAccess)?;
        self.registers.pc = u16::from_le_bytes([lo_pc, hi_pc]);

        Ok(())
    }

    pub fn run(&mut self) -> Result<RunExit, RunError> {
        // Handle events.
        // The real processor can't/won't deaasert these lines.
        if self.reset_pending.load(Ordering::Acquire) {
            self.registers.reset();
            self.jump_indirect(RESET_VECTOR)?;
            self.reset_pending.store(false, Ordering::Release);
        }
        if self.nmi_pending.load(Ordering::Acquire) {
            self.jump_indirect(NMI_VECTOR)?;
            self.nmi_pending.store(false, Ordering::Release);
            return Ok(RunExit::NonMaskableInterrupt);
        }
        if !self.registers.p.contains(Status::INT_DIS) && self.irq_pending.load(Ordering::Acquire) {
            self.jump_indirect(IRQ_VECTOR)?;
            self.irq_pending.store(false, Ordering::Release);
            return Ok(RunExit::Interrupt);
        }

        // Fetch instruction
        let opcode = Opcode::from(
            self.memory
                .read(self.registers.pc)
                .map_err(RunError::CannotFetchInstruction)?,
        );
        let insn = crate::insns::get_insn_by_opcode(opcode.into());
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
            Insn::JAM => return Err(RunError::InvalidInstruction(opcode.into())),
        }?;

        Ok(RunExit::InstructionExecuted(insn))
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
    fn ldy(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("ldy")
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
    fn sty(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("sty")
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
    fn lda(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("lda")
    }

    #[inline]
    fn ora(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("ora")
    }

    #[inline]
    fn sbc(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("sbc")
    }

    #[inline]
    fn sta(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("sta")
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
    fn ldx(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("ldx")
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
    fn stx(&mut self, _addr_mode: AddressMode) -> Result<(), RunError> {
        todo!("stx")
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
