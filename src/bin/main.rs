use ya6502::Memory;

struct RomRam {
    cells: Vec<u8>,
    rom: (u16, u16),
}

impl RomRam {
    fn new() -> Self {
        Self {
            cells: vec![0; 64 * 1024],
            rom: (u16::MAX, 0),
        }
    }
}

impl Memory for RomRam {
    fn write(&mut self, addr: u16, value: u8) -> Result<(), ya6502::MemoryError> {
        if addr as usize > self.cells.len() {
            return Err(ya6502::MemoryError::BadAddress(addr));
        }

        if self.rom.0 >= addr && addr < self.rom.0 + self.rom.1 {
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
    let memory = RomRam::new();
    let mut mos6502 = ya6502::Mos6502::new(&memory);
    mos6502.irq(); // Should be ignored
    mos6502.reset();

    let exit = mos6502.run().unwrap();
    println!("{:#x?}", mos6502.registers());
    println!("{:#x?}", exit);
}
