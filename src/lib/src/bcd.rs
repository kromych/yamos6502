//! Very primitive BCD functions.
//!
//! No checks for the valid BCD represenation and no special types yet.

pub fn u8_to_bcd(u: u8) -> u8 {
    if u < 100 {
        ((u / 10) << 4) | (u % 10)
    } else {
        0x00
    }
}

pub fn bcd_to_u8(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0f)
}
