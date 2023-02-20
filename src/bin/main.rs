use clap::builder::PossibleValue;
use clap::Parser;
use clap::ValueEnum;
use clap_num::maybe_hex;

use yamos6502::Memory;
use yamos6502::RunExit;
use yamos6502::StackWraparound;
use yamos6502::MAX_MEMORY_SIZE;
use yamos6502::RESET_VECTOR;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogLevel {
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn to_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl ValueEnum for LogLevel {
    fn value_variants<'a>() -> &'a [Self] {
        &[LogLevel::Info, LogLevel::Debug, LogLevel::Trace]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            LogLevel::Info => Some(PossibleValue::new("info")),
            LogLevel::Debug => Some(PossibleValue::new("debug")),
            LogLevel::Trace => Some(PossibleValue::new("trace")),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Paths to the files to seed the memory with.
    ///
    /// Format is (path[:load_addr_hex_no_0x],)+, load addresses must increase,
    /// and the loaded files must not overlap.
    mem_file_list: String,
    /// ROM start. Writes into ROM will cause an error.
    #[arg(long, default_value_t = 0xffff, value_parser=maybe_hex::<u16>)]
    rom_start: u16,
    /// Initial program counter.
    #[arg(long, default_value_t = 0x400, value_parser=maybe_hex::<u16>)]
    reset_pc: u16,
    /// Allow stack wraparound.
    #[arg(long, default_value_t = false)]
    stack_wraparound: bool,
    /// Print statistics after execution every `print_stats` instructions.
    #[arg(long, default_value_t = 0)]
    print_stats: u64,
    /// Pause in milliseconds between executing instructions
    #[arg(long)]
    pause_millis: Option<u64>,
    /// Logging level
    #[clap(long, default_value = "info")]
    log: LogLevel,
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
        log::trace!("Wrote 0x{:02x} to 0x{:04x}", value, addr);

        Ok(())
    }

    fn read(&self, addr: u16) -> Result<u8, yamos6502::MemoryError> {
        if addr as usize > self.cells.len() {
            return Err(yamos6502::MemoryError::BadAddress(addr));
        }

        let value = self.cells[addr as usize];
        log::trace!("Read 0x{:02x} from 0x{:04x}", value, addr);

        Ok(value)
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_logger(args.log);

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
            if let Ok(addr) = u16::from_str_radix(addr, 16) {
                if memory.len() > addr as usize {
                    anyhow::bail!("Load addresses must increase");
                }
                // Fill the gap
                memory.extend_from_slice(&vec![0; addr as usize - memory.len()]);
            } else {
                anyhow::bail!(
                    "Load address {} isn't an unadorned 16-bit hex number (0000-ffff)",
                    addr
                );
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

    log::info!("Running emulator");

    mos6502.set_reset_pending();

    let mut instructions_emulated = 0;
    loop {
        let run = mos6502.run();
        match run {
            Ok(RunExit::Executed(insn)) => {
                log::debug!("{insn:?}, {:04x?}", mos6502.registers());
                instructions_emulated += 1;

                if args.print_stats != 0 && instructions_emulated % args.print_stats == 0 {
                    log::info!("Instructions emulated: {instructions_emulated}");
                    log::info!("Last one: {insn:?}, {:04x?}", mos6502.registers());
                }
            }
            Ok(RunExit::Interrupt) => log::debug!("Interrupt {:04x?}", mos6502.registers()),
            Ok(RunExit::NonMaskableInterrupt) => {
                log::debug!("Non-maskable interrupt {:04x?}", mos6502.registers())
            }
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

fn init_logger(log_level: LogLevel) {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or("YAMOS6502_LEVEL", log_level.to_str()),
    );
}
