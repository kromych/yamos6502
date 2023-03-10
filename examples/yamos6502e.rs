use clap::Parser;
use clap_num::maybe_hex;

use yamos6502::Memory;
use yamos6502::RunExit;
use yamos6502::StackWraparound;
use yamos6502::MAX_MEMORY_SIZE;
use yamos6502::RESET_VECTOR;

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
    /// Program counter at which exit
    #[arg(long, default_value_t = 0x3469, value_parser=maybe_hex::<u16>)]
    exit_pc: u16,
    /// Allow stack wraparound.
    #[arg(long, default_value_t = false)]
    stack_wraparound: bool,
    /// Print statistics after execution every `print_stats` instructions.
    #[arg(long, default_value_t = 0)]
    print_stats: u64,
    /// Pause in milliseconds between executing instructions
    #[arg(long)]
    pause_millis: Option<u64>,
    /// Dead loop iterations before exit
    #[clap(long, default_value_t = 0x10000, value_parser=maybe_hex::<u64>)]
    dead_loop_iterations: u64,
    /// Logging level
    #[clap(long, default_value = "info")]
    log_level: log::LevelFilter,
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

    fn read(&mut self, addr: u16) -> Result<u8, yamos6502::MemoryError> {
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
    init_logger(args.log_level);

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

    log::info!("Will exit at 0x{:04x?}", args.exit_pc);

    let memory = RomRam::new(memory, args.rom_start);
    let mut mos6502 = yamos6502::Mos6502::new(memory, allow_stack_wraparound);

    log::info!("Running MOS 6502 emulator");

    mos6502.set_reset_pending();

    let mut instructions_emulated = 0;
    let mut prev_pc = !args.reset_pc;
    let mut dead_loop_iterations = 0;
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

        let pc = mos6502.registers().pc();
        if pc == args.exit_pc {
            log::info!("Exiting as the program is at the exit PC 0x{pc:04x}",);
            log::info!("Instructions emulated: {instructions_emulated}");
            log::info!("{:04x?}", mos6502.registers());
            break;
        }

        if prev_pc == pc {
            dead_loop_iterations += 1;
        } else {
            prev_pc = pc;
            dead_loop_iterations = 0;
        }
        if dead_loop_iterations > args.dead_loop_iterations {
            log::error!(
                "Dead loop with {:04x?} after {instructions_emulated} instructions",
                mos6502.registers()
            );
            anyhow::bail!("Dead loop for {dead_loop_iterations} iterations, aborting");
        }
    }

    Ok(())
}

fn init_logger(level: log::LevelFilter) {
    env_logger::builder()
        .format_timestamp_millis()
        .filter(None, level)
        .init();
}
