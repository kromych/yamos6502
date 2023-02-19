//! Behavioral emulator of MOS 6502

#![no_std]

mod insns;
mod regfile;
mod tests;
mod ya6502;

pub use crate::insns::*;
pub use crate::regfile::*;
pub use crate::ya6502::*;
