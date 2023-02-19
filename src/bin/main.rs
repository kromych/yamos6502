use ya6502::Memory;

struct RomRam {
    cells: Vec<u8>,
    rom_start: u16,
}

impl RomRam {
    fn new() -> Self {
        Self {
            cells: vec![0; 64 * 1024],
            rom_start: 0xe000,
        }
    }
}

impl Memory for RomRam {
    fn write(&mut self, addr: u16, value: u8) -> Result<(), ya6502::MemoryError> {
        if addr as usize > self.cells.len() {
            return Err(ya6502::MemoryError::BadAddress(addr));
        }

        if addr >= self.rom_start {
            return Err(ya6502::MemoryError::ReadOnlyAddress(addr));
        }

        self.cells[addr as usize] = value;
        Ok(())
    }

    fn read(&self, addr: u16) -> Result<u8, ya6502::MemoryError> {
        if addr as usize > self.cells.len() {
            return Err(ya6502::MemoryError::BadAddress(addr));
        }

        Ok(self.cells[addr as usize])
    }
}

fn main() {
    let mut memory = RomRam::new();
    let mut mos6502 = ya6502::Mos6502::new(&mut memory);
    mos6502.irq(); // Should be ignored
    mos6502.reset();

    println!("{:#x?}", mos6502.registers());
    let exit = mos6502.run().unwrap();
    println!("{:#x?}", mos6502.registers());
    println!("{:#x?}", exit);
}
