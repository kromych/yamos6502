use clap::Parser;
use yamos6502::Memory;
use yamos6502::StackWraparound;
use yamos6502::MAX_MEMORY_SIZE;
use yamos6502::RESET_VECTOR;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths to the files to seed the memory with.
    /// Format (path[:load_addr],)+, load addresses must increase.
    mem_file_list: String,
    /// Optional ROM start. Writes into ROM will cause an error.
    #[arg(long, default_value_t = 0xffff)]
    rom_start: u16,
    /// Initial program counter.
    #[arg(long, default_value_t = 0x400)]
    reset_pc: u16,
    /// Allow stack wraparound.
    #[arg(long, default_value_t = true)]
    stack_wraparound: bool,
    /// Pause in milliseconds between executing instructions
    #[arg(long)]
    pause_millis: Option<u64>,
}

struct RomRam {
    cells: Vec<u8>,
    rom_start: u16,
}

impl RomRam {
    fn new(cells: Vec<u8>, rom_start: u16) -> Self {
        Self { cells, rom_start }
    }
}

impl Memory for RomRam {
    fn write(&mut self, addr: u16, value: u8) -> Result<(), yamos6502::MemoryError> {
        if addr as usize > self.cells.len() {
            return Err(yamos6502::MemoryError::BadAddress(addr));
        }

        if addr >= self.rom_start {
            return Err(yamos6502::MemoryError::ReadOnlyAddress(addr));
        }

        self.cells[addr as usize] = value;
        log::info!("Wrote 0x{:02x} to 0x{:04x}", value, addr);

        Ok(())
    }

    fn read(&self, addr: u16) -> Result<u8, yamos6502::MemoryError> {
        if addr as usize > self.cells.len() {
            return Err(yamos6502::MemoryError::BadAddress(addr));
        }

        let value = self.cells[addr as usize];
        log::info!("Read 0x{:02x} from 0x{:04x}", value, addr);

        Ok(value)
    }
}

fn main() -> anyhow::Result<()> {
    init_logger();

    let args = Args::parse();

    let mut memory = vec![];
    for file_path_addr in args.mem_file_list.split(',') {
        let mut file_path_addr = file_path_addr.split(':');

        let file_path = file_path_addr.next();
        if file_path.is_none() {
            anyhow::bail!("Unexpected format of the memory file list");
        }
        let file_path = file_path.unwrap();
        log::info!("Reading memory contents from {file_path}");
        let chunk = std::fs::read(file_path)?;
        log::info!("Read 0x{:04x} bytes", chunk.len());

        if let Some(addr) = file_path_addr.next() {
            if let Ok(addr) = str::parse::<u16>(addr) {
                if memory.len() > addr as usize {
                    anyhow::bail!("Load addresses must increase");
                }
                // Fill the gap
                memory.extend_from_slice(&vec![0; addr as usize - memory.len()]);
            } else {
                anyhow::bail!("Unexpected format of the load address in the memory file list");
            }
        }
        log::info!("Loading at 0x{:04x}", memory.len());
        memory.extend_from_slice(&chunk);
    }

    if memory.len() > MAX_MEMORY_SIZE {
        anyhow::bail!(
            "Loaded 0x{:04x} bytes, maximum memory size is 0x{MAX_MEMORY_SIZE:04x} bytes",
            memory.len()
        );
    }

    // Fill the gap
    memory.extend_from_slice(&vec![0; MAX_MEMORY_SIZE - memory.len()]);

    let allow_stack_wraparound = if args.stack_wraparound {
        StackWraparound::Allow
    } else {
        StackWraparound::Disallow
    };
    log::info!("Stack wraparound policy: {allow_stack_wraparound:?}");

    log::info!("Setting reset vector to 0x{:04x?}", args.reset_pc);
    memory[RESET_VECTOR as usize] = args.reset_pc as u8;
    memory[RESET_VECTOR as usize + 1] = (args.reset_pc >> 8) as u8;

    let mut memory = RomRam::new(memory, args.rom_start);
    let mut mos6502 = yamos6502::Mos6502::new(&mut memory, allow_stack_wraparound);

    mos6502.set_reset_pending();

    loop {
        let run = mos6502.run();
        match run {
            Ok(exit) => log::info!("{:04x?} {:04x?}", exit, mos6502.registers()),
            Err(exit) => {
                log::error!("{:04x?} {:04x?}", exit, mos6502.registers());
                anyhow::bail!("run error");
            }
        }

        if let Some(millis) = args.pause_millis {
            std::thread::sleep(std::time::Duration::from_millis(millis));
        }
    }
}

fn init_logger() {
    env_logger::init_from_env(env_logger::Env::default().filter_or("YAMOS6502_LEVEL", "info"));
}
