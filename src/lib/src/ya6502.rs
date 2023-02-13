//! Behavioral emulator of MOS 6502

use core::fmt::Debug;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::insns::Insn;
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
        // by software
        self.p.set(Status::INT_DIS, true);
        self.p.set(Status::_IGNORED, true);
        self.p.set(Status::BCD, false);

        // Set the stack pointer (the datasheet doesn't mention that,
        // most likely that is handled by boards as a part of reset)
        self.sp = 0xfd;
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

    fn handle_group_00(&mut self, _insn: Insn) -> Result<u16, RunError> {
        Ok(0)
    }

    fn handle_group_01(&mut self, _insn: Insn) -> Result<u16, RunError> {
        Ok(0)
    }

    fn handle_group_10(&mut self, _insn: Insn) -> Result<u16, RunError> {
        Ok(0)
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
        if !insn.is_valid() {
            return Err(RunError::InvalidInstruction(opcode.into()));
        }

        let insn_len = match opcode.insn_group() {
            0b00 => self.handle_group_00(insn)?,
            0b01 => self.handle_group_01(insn)?,
            0b10 => self.handle_group_10(insn)?,
            _ => return Err(RunError::InvalidInstruction(opcode.into())),
        };

        // Advance PC
        self.registers.pc += insn_len;

        if matches!(insn, Insn::BRK) {
            Ok(RunExit::Break)
        } else {
            Ok(RunExit::InstructionExecuted(insn))
        }
    }
}
