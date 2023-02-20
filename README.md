# MOS 6502 Functional Emulator

## What is this?

This emulator is geared towards an easy integration with the `no_std` Rust projects.
That said, it should be very easy to use in the `std` projects, too. A great emphasis
has been put on the clean implementation. In few places some bit twiddling is used
to make the code branchless and hopefully even more performant.

It comes with some unit-tests tests (work in progress), and most notably, the emulator
passes the [6502 functional tests](https://github.com/Klaus2m5/6502_65C02_functional_tests).

This emulator performs computation to the letter of the specification which is stricter
than the real hardware and 6502 netlist-based emulators such as
[Perfect 6502](https://github.com/mist64/perfect6502). In short, this is not a hardware
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

## Examples

### `yamos6502e`

```text
Usage: yamos6502e [OPTIONS] <MEM_FILE_LIST>

Arguments:
  <MEM_FILE_LIST>  Paths to the files to seed the memory with. Format is (path[:load_addr_hex_no_0x],)+, load addresses must increase

Options:
      --rom-start <ROM_START>        ROM start. Writes into ROM will cause an error [default: 65535]
      --reset-pc <RESET_PC>          Initial program counter [default: 1024]
      --stack-wraparound             Allow stack wraparound
      --print-stats <PRINT_STATS>    Print statistics after execution every `print_stats` instructions [default: 0]
      --pause-millis <PAUSE_MILLIS>  Pause in milliseconds between executing instructions
  -h, --help                         Print help
  -V, --version                      Print version
```

To run the 6502 functional tests and print staticstics every 1200000 instructions:

```sh
# Assuming https://github.com/Klaus2m5/6502_65C02_functional_tests is cloned one directory above
cargo run --release ../6502_65C02_functional_tests/bin_files/6502_functional_test.bin --print-stats 1200000
```

## The 6502-related resources and projects I have found inspiration in

### Emulators

1. [6502 functional tests](https://github.com/Klaus2m5/6502_65C02_functional_tests)
2. [Wide NES](https://github.com/daniel5151/ANESE)
3. [My Little 6502](https://github.com/C-Chads/MyLittle6502)
4. [MCS6502](https://github.com/bzotto/MCS6502)

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
