//! Register file
//!
//! MOS 6502 has four 8-bit registers and the 16-bit instruction
//! pointer.

use bitflags::bitflags;
use core::fmt::Debug;

bitflags! {
    /// SR Flags (bit 7 to bit 0)
    pub struct Status : u8 {
        /// N	Negative
        const NEGATIVE = 0x80;
        /// V	Overflow
        const OVERFLOW = 0x40;
        /// -   Ignored (in the register, hardwired to the logic `1`)
        const _IGNORED = 0x20;
        /// B	Break (is never set in the register,
        ///            only in the register value pushed on the stack which
        ///            happens when executing BRK)
        const BRK = 0x10;
        /// D	Decimal (use BCD for arithmetics), cleared on reset
        const DECIMAL = 0x08;
        /// I	Interrupt (IRQ) disable, set on reset
        const INTERRUPT_DISABLE = 0x04;
        /// Z	Zero
        const ZERO = 0x02;
        /// C	Carry
        const CARRY = 0x01;
    }
}

/// Register
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    /// Accumulator
    A = 0,
    /// X index register
    X = 1,
    /// Y index register
    Y = 2,
    /// Status register [NV-BDIZC]
    P = 3,
    /// Stack pointer. The stack grows top-down
    SP = 4,
}

/// MOS 6502 register file
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RegisterFile {
    /// Program counter, little-endian
    pc: u16,
    /// General-purpose registers    
    r: [u8; 5],
}

impl RegisterFile {
    /// Some arbitrary values giving an
    /// incosistent state to catch bugs
    pub fn new() -> Self {
        let r: [u8; 5] = [0xAA, 0xCC, 0xD2, 0, 0x01];
        Self { pc: 0xFF55, r }
    }

    pub fn reset(&mut self) {
        // Hardware sets few flags, everything else is initialized
        // by software.
        self.set_flag(Status::INTERRUPT_DISABLE);
        self.set_flag(Status::_IGNORED);
        self.clear_flag(Status::DECIMAL);

        // Stack pointer is not set! In some configurations that might
        // not even be useful, e.g. if the only type of memory is ROM.
        // The software is expected to initialize the stack pointer to
        // use interrupts and subroutine calls.
    }

    #[inline]
    pub fn reg_mut(&mut self, reg: Register) -> &mut u8 {
        &mut self.r[(reg as u8) as usize]
    }

    #[inline]
    pub fn reg(&self, reg: Register) -> u8 {
        self.r[(reg as u8) as usize]
    }

    #[inline]
    pub fn a(&self) -> u8 {
        self.reg(Register::A)
    }

    #[inline]
    pub fn x(&self) -> u8 {
        self.reg(Register::X)
    }

    #[inline]
    pub fn y(&self) -> u8 {
        self.reg(Register::Y)
    }

    #[inline]
    pub fn sp(&self) -> u8 {
        self.reg(Register::SP)
    }

    #[inline]
    pub fn pc(&self) -> u16 {
        self.pc
    }

    #[inline]
    pub fn a_mut(&mut self) -> &mut u8 {
        &mut self.r[(Register::A as u8) as usize]
    }

    #[inline]
    pub fn x_mut(&mut self) -> &mut u8 {
        &mut self.r[(Register::X as u8) as usize]
    }

    #[inline]
    pub fn y_mut(&mut self) -> &mut u8 {
        &mut self.r[(Register::Y as u8) as usize]
    }

    #[inline]
    pub fn sp_mut(&mut self) -> &mut u8 {
        &mut self.r[(Register::SP as u8) as usize]
    }

    #[inline]
    pub fn pc_mut(&mut self) -> &mut u16 {
        &mut self.pc
    }

    #[inline]
    pub fn flag_set(&self, flag: Status) -> bool {
        (self.reg(Register::P) & flag.bits) != 0
    }

    #[inline]
    pub fn set_flag(&mut self, flag: Status) {
        *self.reg_mut(Register::P) |= flag.bits;
    }

    #[inline]
    pub fn clear_flag(&mut self, flag: Status) {
        *self.reg_mut(Register::P) &= !flag.bits;
    }
}

impl Default for RegisterFile {
    fn default() -> Self {
        RegisterFile::new()
    }
}

impl Debug for RegisterFile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RegisterFile")
            .field("PC", &self.pc)
            .field("A", &self.a())
            .field("X", &self.x())
            .field("Y", &self.y())
            .field("SP", &self.sp())
            .field("P", &Status::from_bits(self.r[Register::P as u8 as usize]))
            .finish()
    }
}
