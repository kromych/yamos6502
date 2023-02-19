//! Behavioral emulator of MOS 6502
//!
//! There is no cycle-accurate emulation, support
//! for undocumented instructions and other microarch
//! side effects such as writing the old value first
//! for the read-modify-write instructions.
//!
//! Unsuported instructions result in the execution jam,
//! and the processor rolls its state back to the previous
//! instruction.

use core::fmt::Debug;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::insns::Insn;
use crate::AddressMode;
use crate::Register;
use crate::RegisterFile;
use crate::Status;

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

/// When an interrupt is signaled, the low and the high
/// 8 bits of the program counter are loaded
/// from these addresses.
pub const IRQ_VECTOR: u16 = 0xFFFE;

/// When a reset is requested, the low and the high
/// 8 bits of the program counter are loaded
/// from these addresses.
pub const RESET_VECTOR: u16 = 0xFFFC;

/// When an non-maskable interrupt is signaled,
/// the low and the high 8 bits of the program counter
/// are loaded from these addresses.
pub const NMI_VECTOR: u16 = 0xFFFA;

/// Run normal exit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunExit {
    /// Instruction retired
    InstructionExecuted(Insn),
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
    reg_file: RegisterFile,
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
            reg_file: RegisterFile::default(),
            reset_pending: AtomicBool::new(false),
            nmi_pending: AtomicBool::new(false),
            irq_pending: AtomicBool::new(false),
            fault: None,
            last_opcode: 0,
        }
    }

    pub fn with_registers(memory: &'memory mut M, regf: RegisterFile) -> Self {
        Self {
            mem: memory,
            reg_file: regf,
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

    pub fn registers(&self) -> &RegisterFile {
        &self.reg_file
    }

    fn read_u8(&self, addr: u16) -> Result<u8, RunError> {
        self.mem.read(addr).map_err(RunError::MemoryAccess)
    }

    fn write_u8(&mut self, addr: u16, value: u8) -> Result<(), RunError> {
        self.mem.write(addr, value).map_err(RunError::MemoryAccess)
    }

    fn load_u16(&self, addr: u16) -> Result<u16, RunError> {
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
            AddressMode::Immediate | AddressMode::Relative => {
                let ea = self.reg_file.pc();
                self.reg_file.adjust_pc_by(1);

                Ok(ea)
            }
            AddressMode::Indirect => {
                let ptr = self.load_u16(self.reg_file.pc())?;
                self.reg_file.adjust_pc_by(2);
                let ea = self.load_u16(ptr)?;

                Ok(ea)
            }
            AddressMode::Xindirect => {
                let ptr = self
                    .read_u8(self.reg_file.pc())?
                    .wrapping_add(self.reg_file.x())
                    .into();
                self.reg_file.adjust_pc_by(1);
                let ea = self.load_u16(ptr)?;

                Ok(ea)
            }
            AddressMode::IndirectY => {
                let ptr = self.read_u8(self.reg_file.pc())?.into();
                self.reg_file.adjust_pc_by(1);
                let ea = self.load_u16(ptr)?.wrapping_add(self.reg_file.y().into());

                Ok(ea)
            }
            AddressMode::Absolute => {
                let ea = self.load_u16(self.reg_file.pc())?;
                self.reg_file.adjust_pc_by(2);

                Ok(ea)
            }
            AddressMode::AbsoluteX => {
                let ea = self
                    .load_u16(self.reg_file.pc())?
                    .wrapping_add(self.reg_file.x().into());
                self.reg_file.adjust_pc_by(2);

                Ok(ea)
            }
            AddressMode::AbsoluteY => {
                let ea = self
                    .load_u16(self.reg_file.pc())?
                    .wrapping_add(self.reg_file.y().into());
                self.reg_file.adjust_pc_by(2);

                Ok(ea)
            }
            AddressMode::Zeropage => {
                let ea = self.read_u8(self.reg_file.pc())?.into();
                self.reg_file.adjust_pc_by(1);

                Ok(ea)
            }
            AddressMode::ZeropageX => {
                let ea = self
                    .read_u8(self.reg_file.pc())?
                    .wrapping_add(self.reg_file.x())
                    .into();
                self.reg_file.adjust_pc_by(1);

                Ok(ea)
            }
            AddressMode::ZeropageY => {
                let ea = self
                    .read_u8(self.reg_file.pc())?
                    .wrapping_add(self.reg_file.y())
                    .into();
                self.reg_file.adjust_pc_by(1);

                Ok(ea)
            }
        }
    }

    #[inline]
    fn update_flags_nz(&mut self, data: u8) {
        self.reg_file.set_flag_from_cond(Status::Zero, data == 0);
        self.reg_file
            .set_flag_from_cond(Status::Negative, (data as i8) < 0);
    }

    #[inline]
    fn read_modify_to_reg<F>(
        &mut self,
        addr_mode: AddressMode,
        reg: Register,
        modify: F,
    ) -> Result<u8, RunError>
    where
        F: Fn(u8) -> u8,
    {
        let ea = self.get_effective_address(addr_mode)?;
        let value = self.read_u8(ea)?;
        let value = modify(value);
        *self.reg_file.reg_mut(reg) = value;
        self.update_flags_nz(value);

        Ok(value)
    }

    #[inline]
    fn mem_to_reg(&mut self, addr_mode: AddressMode, reg: Register) -> Result<u8, RunError> {
        self.read_modify_to_reg(addr_mode, reg, |v| v)
    }

    #[inline]
    fn reg_to_mem(&mut self, reg: Register, addr_mode: AddressMode) -> Result<u8, RunError> {
        let ea = self.get_effective_address(addr_mode)?;
        let value = self.reg_file.reg(reg);
        self.write_u8(ea, value)?;

        Ok(value)
    }

    #[inline]
    fn reg_to_reg(&mut self, reg_src: Register, reg_dst: Register) -> u8 {
        *self.reg_file.reg_mut(reg_dst) = self.reg_file.reg(reg_src);
        let value = self.reg_file.reg(reg_dst);
        self.update_flags_nz(value);

        value
    }

    fn read_modify_write_mem<F>(
        &mut self,
        addr_mode: AddressMode,
        modify: F,
    ) -> Result<u8, RunError>
    where
        F: Fn(u8) -> u8,
    {
        let ea = self.get_effective_address(addr_mode)?;
        let value = self.read_u8(ea)?;
        let value = modify(value);
        self.write_u8(ea, value)?;
        self.update_flags_nz(value);

        Ok(value)
    }

    fn read_modify_write_reg<F>(&mut self, reg: Register, modify: F) -> u8
    where
        F: Fn(u8) -> u8,
    {
        let value = self.reg_file.reg(reg);
        let value = modify(value);
        *self.reg_file.reg_mut(reg) = value;
        self.update_flags_nz(value);

        value
    }

    fn flag_set(&self, flag: Status) -> bool {
        self.reg_file.flag_set(flag)
    }

    fn branch(&mut self, addr_mode: AddressMode, cond: bool) -> Result<(), RunError> {
        if cond {
            let ea = self.get_effective_address(addr_mode)?;
            let offset = self.read_u8(ea)? as i8;
            self.reg_file.adjust_pc_by(offset);
        }
        Ok(())
    }

    fn step(&mut self) -> Result<RunExit, RunError> {
        // Fetch instruction
        self.last_opcode = self
            .mem
            .read(self.reg_file.pc())
            .map_err(RunError::CannotFetchInstruction)?;
        self.reg_file.adjust_pc_by(1);

        // Decode instruction from the opcode
        let insn = crate::insns::decode_insn(self.last_opcode);

        // Execute instruction
        match insn {
            // Group 0b00. Flags, conditionals, jumps, misc. There are a few
            // quite complex instructions here.
            Insn::BRK => todo!("brk"),
            Insn::BCC(addr_mode) => self.branch(addr_mode, !self.flag_set(Status::Carry))?,
            Insn::BCS(addr_mode) => self.branch(addr_mode, self.flag_set(Status::Carry))?,
            Insn::BNE(addr_mode) => self.branch(addr_mode, !self.flag_set(Status::Zero))?,
            Insn::BEQ(addr_mode) => self.branch(addr_mode, self.flag_set(Status::Zero))?,
            Insn::BVC(addr_mode) => self.branch(addr_mode, !self.flag_set(Status::Overflow))?,
            Insn::BVS(addr_mode) => self.branch(addr_mode, self.flag_set(Status::Overflow))?,
            Insn::BPL(addr_mode) => self.branch(addr_mode, !self.flag_set(Status::Negative))?,
            Insn::BMI(addr_mode) => self.branch(addr_mode, self.flag_set(Status::Negative))?,
            Insn::BIT(_addr_mode) => todo!("bit"),
            Insn::CLC => self.reg_file.clear_flag(Status::Carry),
            Insn::CLD => self.reg_file.clear_flag(Status::Decimal),
            Insn::CLI => self.reg_file.clear_flag(Status::InterruptDisable),
            Insn::CLV => self.reg_file.clear_flag(Status::Overflow),
            Insn::CPX(_addr_mode) => todo!("cpx"),
            Insn::CPY(_addr_mode) => todo!("cpy"),
            Insn::DEY => {
                self.read_modify_write_reg(Register::Y, |v| v.wrapping_sub(1));
            }
            Insn::INX => {
                self.read_modify_write_reg(Register::X, |v| v.wrapping_add(1));
            }
            Insn::INY => {
                self.read_modify_write_reg(Register::Y, |v| v.wrapping_add(1));
            }
            Insn::JMP(addr_mode) => {
                let pc_ptr = self.get_effective_address(addr_mode)?;
                self.reg_file.set_pc(self.load_u16(pc_ptr)?);
            }
            Insn::JSR(_addr_mode) => todo!("jsr"),
            Insn::LDY(addr_mode) => {
                self.mem_to_reg(addr_mode, Register::Y)?;
            }
            Insn::PHA => todo!("pha"),
            Insn::PHP => todo!("php"),
            Insn::PLA => todo!("pla"),
            Insn::PLP => todo!("plp"),
            Insn::RTI => todo!("rti"),
            Insn::RTS => todo!("rts"),
            Insn::SEC => self.reg_file.set_flag(Status::Carry),
            Insn::SED => self.reg_file.set_flag(Status::Decimal),
            Insn::SEI => self.reg_file.set_flag(Status::InterruptDisable),
            Insn::STY(addr_mode) => {
                self.reg_to_mem(Register::Y, addr_mode)?;
            }
            Insn::TAY => {
                self.reg_to_reg(Register::A, Register::Y);
            }
            Insn::TYA => {
                self.reg_to_reg(Register::Y, Register::A);
            }
            // Group 0b01. ALU instructions and load/store for the accumulator.
            // Very regular encoding to make decoding and execution for the common path
            // faster in hardware (presumably).
            Insn::ADC(_addr_mode) => todo!("adc"),
            Insn::AND(addr_mode) => {
                let a = self.reg_file.a();
                self.read_modify_to_reg(addr_mode, Register::A, |v| v & a)?;
            }
            Insn::CMP(_addr_mode) => todo!("cmp"),
            Insn::EOR(addr_mode) => {
                let a = self.reg_file.a();
                self.read_modify_to_reg(addr_mode, Register::A, |v| v ^ a)?;
            }
            Insn::LDA(addr_mode) => {
                self.mem_to_reg(addr_mode, Register::A)?;
            }
            Insn::ORA(addr_mode) => {
                let a = self.reg_file.a();
                self.read_modify_to_reg(addr_mode, Register::A, |v| v | a)?;
            }
            Insn::SBC(_addr_mode) => todo!("sbc"),
            Insn::STA(addr_mode) => {
                self.reg_to_mem(Register::A, addr_mode)?;
            }
            // Group 0b10. Bit operation and accumulator operations,
            // less regular than the ALU group.
            Insn::ASLA => todo!("asl a"),
            Insn::ASL(_addr_mode) => todo!("asl"),
            Insn::DEC(addr_mode) => {
                self.read_modify_write_mem(addr_mode, |v| v.wrapping_sub(1))?;
            }
            Insn::DEX => {
                self.read_modify_write_reg(Register::X, |v| v.wrapping_sub(1));
            }
            Insn::INC(addr_mode) => {
                self.read_modify_write_mem(addr_mode, |v| v.wrapping_add(1))?;
            }
            Insn::LDX(addr_mode) => {
                self.mem_to_reg(addr_mode, Register::X)?;
            }
            Insn::LSRA => todo!("lsr a"),
            Insn::LSR(_addr_mode) => todo!("lsr"),
            Insn::NOP => {}
            Insn::ROLA => todo!("rol a"),
            Insn::ROL(_addr_mode) => todo!("rol"),
            Insn::RORA => todo!("ror a"),
            Insn::ROR(_addr_mode) => todo!("ror"),
            Insn::STX(addr_mode) => {
                self.reg_to_mem(Register::X, addr_mode)?;
            }
            Insn::TAX => {
                self.reg_to_reg(Register::A, Register::X);
            }
            Insn::TSX => {
                self.reg_to_reg(Register::S, Register::X);
            }
            Insn::TXA => {
                self.reg_to_reg(Register::X, Register::A);
            }
            Insn::TXS => {
                self.reg_to_reg(Register::X, Register::S);
            }
            // Group 0b11 contains invalid instructions
            Insn::JAM => {
                return Err(RunError::InvalidInstruction(self.last_opcode));
            }
        };

        Ok(RunExit::InstructionExecuted(insn))
    }

    pub fn run(&mut self) -> Result<RunExit, RunError> {
        // Handle reset.
        // The real processor can't/won't deaasert the line.
        if self.reset_pending.load(Ordering::Acquire) {
            self.reg_file.set_pc(self.load_u16(RESET_VECTOR)?);
            self.fault = None;
            self.reg_file.reset();
            self.reset_pending.store(false, Ordering::Release);
        }

        // If the processor faulted, refuse to run.
        if let Some(f) = self.fault {
            return Err(f);
        }

        // Handle other events.
        // The real processor can't/won't deaasert these lines.
        if self.nmi_pending.load(Ordering::Acquire) {
            self.reg_file.set_pc(self.load_u16(NMI_VECTOR)?);
            self.nmi_pending.store(false, Ordering::Release);

            return Ok(RunExit::NonMaskableInterrupt);
        }
        if !self.flag_set(Status::InterruptDisable) && self.irq_pending.load(Ordering::Acquire) {
            self.reg_file.set_pc(self.load_u16(IRQ_VECTOR)?);
            self.irq_pending.store(false, Ordering::Release);

            return Ok(RunExit::Interrupt);
        }

        // The register state is rolled back on an instruction fault
        let registers = self.reg_file;
        match self.step() {
            Err(e) => {
                self.fault = Some(e);
                self.reg_file = registers;
                Err(e)
            }
            Ok(o) => Ok(o),
        }
    }
}
