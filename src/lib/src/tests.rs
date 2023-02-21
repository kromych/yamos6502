#![cfg(test)]

use crate::*;

struct TestMemory {
    bytes: [u8; u16::MAX as usize],
}

impl TestMemory {
    fn write_u16(&mut self, addr: u16, value: u16) {
        let addr = addr as usize;
        self.bytes[addr] = value as u8;
        self.bytes[addr + 1] = (value >> 8) as u8;
    }

    fn write_u8(&mut self, addr: u16, value: u8) {
        let addr = addr as usize;
        self.bytes[addr] = value;
    }

    fn write(&mut self, addr: u16, bytes: &[u8]) {
        let addr = addr as usize;
        for (i, &b) in bytes.iter().enumerate() {
            self.bytes[addr.wrapping_add(i)] = b;
        }
    }
}

impl Default for TestMemory {
    fn default() -> Self {
        Self {
            bytes: [0x55; u16::MAX as usize],
        }
    }
}

impl Memory for TestMemory {
    fn write(&mut self, addr: u16, value: u8) -> Result<(), crate::MemoryError> {
        Ok(self.bytes[addr as usize] = value)
    }

    fn read(&self, addr: u16) -> Result<u8, crate::MemoryError> {
        Ok(self.bytes[addr as usize])
    }
}

const TEST_START: u16 = 0x0200;
const ABSOLUTE_START: u16 = 0x1200;

#[test]
fn test_loads() {
    let mut memory = TestMemory::default();

    // Set up the reset address to point to the test program
    memory.write_u16(RESET_VECTOR, TEST_START);

    // Some data in the memory
    memory.write_u8(ABSOLUTE_START, 0xab);
    memory.write_u8(ABSOLUTE_START + 0xf3, 0xac);
    memory.write_u8(ABSOLUTE_START + 0xf4, 0xad);
    memory.write_u8(ABSOLUTE_START + 0xcc, 0xae);

    // Some data on the zero page
    memory.write_u8(0x35, 0xba);
    memory.write_u8(0x36, 0xbb);
    memory.write_u8(0x37, 0xcc);
    memory.write_u8(0x43, 0xbd);
    memory.write_u8(0x44, 0xbe);

    // Data pointed to by the previous with X-indirect (note wraparound)
    // and indirect-Y addressing
    memory.write_u8(0xbbba, 0x77);
    memory.write_u8(0xbebd, 0x74);
    memory.write_u8(0xbfb1, 0x78);

    memory.write_u16(0x000f, 0xbbba);

    let program = [
        encode_insn(Insn::LDA(AddressMode::Immediate)),
        0x12,
        encode_insn(Insn::LDA(AddressMode::Immediate)),
        0x00,
        encode_insn(Insn::LDA(AddressMode::Immediate)),
        0xF2,
        encode_insn(Insn::LDX(AddressMode::Immediate)),
        0x13,
        encode_insn(Insn::LDX(AddressMode::Immediate)),
        0x00,
        encode_insn(Insn::LDX(AddressMode::Immediate)),
        0xF3,
        encode_insn(Insn::LDY(AddressMode::Immediate)),
        0x14,
        encode_insn(Insn::LDY(AddressMode::Immediate)),
        0x00,
        encode_insn(Insn::LDY(AddressMode::Immediate)),
        0xF4,
        encode_insn(Insn::LDA(AddressMode::Absolute)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::LDA(AddressMode::AbsoluteX)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::LDA(AddressMode::AbsoluteY)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::LDA(AddressMode::Xindirect)),
        0x42,
        encode_insn(Insn::LDA(AddressMode::IndirectY)),
        0x43,
        encode_insn(Insn::LDA(AddressMode::Zeropage)),
        0x43,
        encode_insn(Insn::LDA(AddressMode::ZeropageX)),
        0x42,
        encode_insn(Insn::LDX(AddressMode::Absolute)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::LDX(AddressMode::AbsoluteY)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::LDX(AddressMode::Zeropage)),
        0x43,
        encode_insn(Insn::LDX(AddressMode::ZeropageY)),
        0x43,
        encode_insn(Insn::LDY(AddressMode::Absolute)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::LDY(AddressMode::AbsoluteX)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::LDY(AddressMode::Zeropage)),
        0x43,
        encode_insn(Insn::LDY(AddressMode::ZeropageX)),
        0x43,
    ];

    // Write the program to the memory
    memory.write(TEST_START, &program);

    let mut mos6502 = Mos6502::new(&mut memory, StackWraparound::Disallow);
    mos6502.set_reset_pending();

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Immediate)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x12);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Immediate)));
    assert!(mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Immediate)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0xf2);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDX(AddressMode::Immediate)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().x() == 0x13);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDX(AddressMode::Immediate)));
    assert!(mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().x() == 0);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDX(AddressMode::Immediate)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().x() == 0xf3);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDY(AddressMode::Immediate)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().y() == 0x14);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDY(AddressMode::Immediate)));
    assert!(mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().y() == 0);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDY(AddressMode::Immediate)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().y() == 0xf4);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Absolute)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0xab);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::AbsoluteX)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0xac);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::AbsoluteY)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0xad);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Xindirect)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x77);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::IndirectY)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x78);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Zeropage)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0xbd);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::ZeropageX)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0xba);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDX(AddressMode::Absolute)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().x() == 0xab);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDX(AddressMode::AbsoluteY)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().x() == 0xad);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDX(AddressMode::Zeropage)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().x() == 0xbd);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDX(AddressMode::ZeropageY)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().x() == 0xcc);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDY(AddressMode::Absolute)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().y() == 0xab);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDY(AddressMode::AbsoluteX)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().y() == 0xae);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDY(AddressMode::Zeropage)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().y() == 0xbd);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDY(AddressMode::ZeropageX)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().y() == 0xba);
}

#[test]
fn test_stores() {
    let mut memory = TestMemory::default();

    let program = [
        encode_insn(Insn::STA(AddressMode::Absolute)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::STA(AddressMode::AbsoluteX)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        encode_insn(Insn::STA(AddressMode::AbsoluteY)),
        ABSOLUTE_START as u8,
        (ABSOLUTE_START >> 8) as u8,
        // get_insn_opcode(Insn::STA(AddressMode::IndirectY)),
        // get_insn_opcode(Insn::STA(AddressMode::Xindirect)),
        // get_insn_opcode(Insn::STA(AddressMode::Zeropage)),
        // get_insn_opcode(Insn::STA(AddressMode::ZeropageX)),
        // get_insn_opcode(Insn::STX(AddressMode::Absolute)),
        // get_insn_opcode(Insn::STX(AddressMode::Zeropage)),
        // get_insn_opcode(Insn::STX(AddressMode::ZeropageY)),
        // get_insn_opcode(Insn::STY(AddressMode::Absolute)),
        // get_insn_opcode(Insn::STY(AddressMode::Zeropage)),
        // get_insn_opcode(Insn::STY(AddressMode::ZeropageX)),
    ];

    // Write the program to the memory
    memory.write(TEST_START, &program);

    {
        let mut regf = RegisterFile::default();
        *regf.a_mut() = 0xaf;
        *regf.x_mut() = 0x1b;
        *regf.y_mut() = 0x2c;
        regf.set_pc(TEST_START);

        let mut mos6502 = Mos6502::with_registers(&mut memory, regf, StackWraparound::Disallow);

        assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::STA(AddressMode::Absolute)));
        assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::STA(AddressMode::AbsoluteX)));
        assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::STA(AddressMode::AbsoluteY)));
    }

    assert!(memory.read(ABSOLUTE_START).unwrap() == 0xaf);
    assert!(memory.read(ABSOLUTE_START + 0x1b).unwrap() == 0xaf);
    assert!(memory.read(ABSOLUTE_START + 0x2c).unwrap() == 0xaf);
}

#[test]
fn test_alu() {
    let mut memory = TestMemory::default();

    let program = [
        encode_insn(Insn::LDA(AddressMode::Immediate)),
        0x00,
        encode_insn(Insn::STA(AddressMode::Zeropage)),
        0x12,
        encode_insn(Insn::CLC),
        encode_insn(Insn::CLD),
        encode_insn(Insn::LDA(AddressMode::Immediate)),
        0x90,
        encode_insn(Insn::ADC(AddressMode::Zeropage)),
        0x12,
        encode_insn(Insn::SBC(AddressMode::Zeropage)),
        0x12,
        encode_insn(Insn::CLC),
        encode_insn(Insn::SED),
        encode_insn(Insn::LDA(AddressMode::Immediate)),
        0x90,
        encode_insn(Insn::ADC(AddressMode::Zeropage)),
        0x12,
        encode_insn(Insn::SBC(AddressMode::Zeropage)),
        0x12,
    ];

    // Write the program to the memory
    memory.write(TEST_START, &program);

    let mut regf = RegisterFile::default();
    regf.set_pc(TEST_START);

    let mut mos6502 = Mos6502::with_registers(&mut memory, regf, StackWraparound::Disallow);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Immediate)));
    assert!(mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::STA(AddressMode::Zeropage)));

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::CLC));
    assert!(!mos6502.registers().flag_set(Status::Carry));
    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::CLD));
    assert!(!mos6502.registers().flag_set(Status::Decimal));
    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Immediate)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x90);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::ADC(AddressMode::Zeropage)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Carry));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x90);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::SBC(AddressMode::Zeropage)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Carry));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x8f);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::CLC));
    assert!(!mos6502.registers().flag_set(Status::Carry));
    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::SED));
    assert!(mos6502.registers().flag_set(Status::Decimal));
    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::LDA(AddressMode::Immediate)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x90);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::ADC(AddressMode::Zeropage)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(!mos6502.registers().flag_set(Status::Carry));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x90);

    assert!(mos6502.run().unwrap() == RunExit::Executed(Insn::SBC(AddressMode::Zeropage)));
    assert!(!mos6502.registers().flag_set(Status::Zero));
    assert!(mos6502.registers().flag_set(Status::Carry));
    assert!(mos6502.registers().flag_set(Status::Negative));
    assert!(mos6502.registers().a() == 0x89);
}

#[test]
fn test_insn() {
    for g in 0..3_u8 {
        for h in 0..8 {
            for l in 0..8 {
                let opcode = (h << 5) | (l << 2) | g;
                let insn = decode_insn(opcode);
                if insn.is_valid() {
                    let same_opcode = encode_insn(insn);
                    assert!(same_opcode == opcode);
                }
            }
        }
    }
}
