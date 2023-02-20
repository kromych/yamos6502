//! Register file
//!
//! MOS 6502 has four 8-bit registers and the 16-bit instruction
//! pointer.

use core::fmt::Debug;

/// SR Flags (bit 7 to bit 0)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// N	Negative
    Negative = 7,
    /// V	Overflow
    Overflow = 6,
    /// -   Ignored (in the register, hardwired to the logic `1`)
    AlwaysSet = 5,
    /// B	Break (is never set in the register,
    ///            only in the register value pushed on the stack which
    ///            happens when executing BRK)
    Break = 4,
    /// D	Decimal (use BCD for arithmetics), cleared on reset
    Decimal = 3,
    /// I	Interrupt (IRQ) disable, set on reset
    InterruptDisable = 2,
    /// Z	Zero
    Zero = 1,
    /// C	Carry
    Carry = 0,
}

impl Status {
    pub fn mask(&self) -> u8 {
        1 << (*self as u8)
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
    S = 4,
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
        *self.reg_mut(Register::P) = 0;
        self.set_flag(Status::InterruptDisable);
        self.set_flag(Status::AlwaysSet);

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
        self.reg(Register::S)
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
        &mut self.r[(Register::S as u8) as usize]
    }

    #[inline]
    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc
    }

    #[inline]
    pub fn adjust_pc_by(&mut self, offset: i8) {
        self.pc = self.pc.wrapping_add(offset as i16 as u16).into();
    }

    #[inline]
    pub fn flag_set(&self, flag: Status) -> bool {
        (self.reg(Register::P) & flag.mask()) != 0
    }

    #[inline]
    pub fn set_flag_from_cond(&mut self, flag: Status, cond: bool) {
        let p = self.reg_mut(Register::P);
        *p = (*p & !flag.mask()) | (cond as u8) << (flag as u8);
    }

    #[inline]
    pub fn set_flag(&mut self, flag: Status) {
        self.set_flag_from_cond(flag, true)
    }

    #[inline]
    pub fn clear_flag(&mut self, flag: Status) {
        self.set_flag_from_cond(flag, false)
    }
}

impl Default for RegisterFile {
    fn default() -> Self {
        RegisterFile::new()
    }
}

impl Debug for RegisterFile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut flags = ['n', 'v', 'x', 'b', 'd', 'i', 'z', 'c'];

        if self.flag_set(Status::Negative) {
            flags[7 - (Status::Negative as u8 as usize)] = 'N';
        }
        if self.flag_set(Status::Overflow) {
            flags[7 - (Status::Overflow as u8 as usize)] = 'V';
        }
        if self.flag_set(Status::AlwaysSet) {
            flags[7 - (Status::AlwaysSet as u8 as usize)] = 'X';
        }
        if self.flag_set(Status::Break) {
            flags[7 - (Status::Break as u8 as usize)] = 'B';
        }
        if self.flag_set(Status::Decimal) {
            flags[7 - (Status::Decimal as u8 as usize)] = 'D';
        }
        if self.flag_set(Status::InterruptDisable) {
            flags[7 - (Status::InterruptDisable as u8 as usize)] = 'I';
        }
        if self.flag_set(Status::Zero) {
            flags[7 - (Status::Zero as u8 as usize)] = 'Z';
        }
        if self.flag_set(Status::Carry) {
            flags[7 - (Status::Carry as u8 as usize)] = 'C';
        }

        f.debug_struct("RegisterFile")
            .field("PC", &self.pc)
            .field("A", &self.a())
            .field("X", &self.x())
            .field("Y", &self.y())
            .field("SP", &self.sp())
            .field("P", &flags)
            .finish()
    }
}
