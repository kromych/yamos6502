# MOS 6502 Functional Emulator

## What is this?

This emulator is geared towards an easy integration with the `no_std` Rust projects
to run in the environment where is no OS. That said, it should be very easy to use
in the `std` projects, too, as demonstrated by the examples below. A great emphasis
has been put on the clean implementation. Some bit twiddling has been employed in
few places to make the code branchless and hopefully even more performant.

The emulator comes with some unit-tests tests (work in progress), and most notably,
it passes the [6502 functional tests](https://github.com/Klaus2m5/6502_65C02_functional_tests).

This emulator strives to perform computation to the letter of the specification which
is stricter than the real hardware and 6502 netlist-based emulators such as
[Perfect 6502](https://github.com/mist64/perfect6502) do. In short, this is not a hardware
simulator.

The emulation algorithm is quite simple:

* fetch the opcode byte
* decode an instrunction based on the breaking the opcode down to the operation
  bits, address mode, and the operation group
* compute the effective address if needed
* emulate the operation, update the state
* adjust the program counter

That said, there is no emulation of the microarch layer, e.g., no:

* cycle-accurate emulation or cycle counting,
* (unintended) support for the "undocumented" instructions. These instructions
  result in the execution jam, and the emulator will roll its state back to the
  previous instruction returning an error on any subsequent invocation until
  it is reset.
* (unintended?) microarch side effects such as (not limited to):
    1. writing the old value first for the read-modify-write instructions,
    2. interrupt hijacking.

Stack underflows and overflows might be set to result in a fault, and the emulator
will not continue execution until it is reset.

It goes without saying that the glitches at the physical layer are not emulated,
too :)

## Usage

Add

```toml
yamos6502 = "0.1"
```

to your project's `Cargo.toml`. If you intend not to depend on the `std` crate, here is the syntax to use instead:

```toml
yamos6502 = { version = "0.1", default_features = false }
```

## Examples

### `yamos6502e`

```text
Usage: yamos6502e [OPTIONS] <MEM_FILE_LIST>

Arguments:
  <MEM_FILE_LIST>
          Paths to the files to seed the memory with.
          Format is (path[:load_addr_hex_no_0x],)+, load addresses must increase, and the loaded files must not overlap.

Options:
      --rom-start <ROM_START>
          ROM start. Writes into ROM will cause an error.
          [default: 65535]
      --reset-pc <RESET_PC>
          Initial program counter
          [default: 1024]
      --exit-pc <EXIT_PC>
          Program counter at which exit
          [default: 13417]
      --stack-wraparound
          Allow stack wraparound
      --print-stats <PRINT_STATS>
          Print statistics after execution every `print_stats` instructions
          [default: 0]
      --pause-millis <PAUSE_MILLIS>
          Pause in milliseconds between executing instructions
      --dead-loop-iterations <DEAD_LOOP_ITERATIONS>
          Dead loop iterations before exit          
          [default: 65536]          
      --log <LOG>
          Logging level          
          [default: info]
          [possible values: info, debug, trace]

  -h, --help
          Print help (see a summary with '-h')
  -V, --version
          Print version
```

To run the 6502 functional tests and print staticstics every 15_000_000 instructions:

```sh
#
# Assuming https://github.com/Klaus2m5/6502_65C02_functional_tests is cloned one directory above:
# 
# `git clone https://github.com/Klaus2m5/6502_65C02_functional_tests ../6502_65C02_functional_tests
#
cargo run --example yamos6502e  \
    ../6502_65C02_functional_tests/bin_files/6502_functional_test.bin:0000 \
    --print-stats 15000000 \
    --reset-pc 0x400 \
    --exit-pc 0x3469 \
    --stack-wraparound \
    --dead-loop-iterations 16
```

Building with `--release` produces a much faster emulator at the cost of omitting some runtime
checks.

## The 6502-related resources and projects I have found inspiration in

### Emulators

1. [6502 functional tests](https://github.com/Klaus2m5/6502_65C02_functional_tests)
2. [Wide NES](https://github.com/daniel5151/ANESE)
3. [My Little 6502](https://github.com/C-Chads/MyLittle6502)
4. [MCS6502](https://github.com/bzotto/MCS6502)
5. [6502](https://github.com/jefftranter/6502)

### Hardware simulators

1. [Visual 6502 Remix](https://floooh.github.io/visual6502remix/)
2. [Perfect 6502](https://github.com/mist64/perfect6502)
3. [Monster 6502](https://monster6502.com/)
4. [Verilog 6502](http://www.aholme.co.uk/6502/Main.htm)

### Compilers, assemblers

1. [ASM X](https://github.com/db-electronics/asmx)
2. [CC65 and CA65](https://github.com/cc65/cc65)
3. [LLVM-MOS](https://github.com/llvm-mos/llvm-mos)

### Documentation, reference materials, RE

1. [Western Design Center](https://www.westerndesigncenter.com/)
2. [6502 Instruction Set](https://www.masswerk.at/6502/6502_instruction_set.html)
3. [6502.org](http://6502.org/)
4. [The 6502 overflow flag explained](https://www.righto.com/2012/12/the-6502-overflow-flag-explained.html)
5. [Fast Binary-Coded Decimal Addition and Subtraction](https://tavianator.com/2011/bcd.html)
