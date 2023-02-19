# MOS 6502 Functional Emulator

## What is this?

This emulator is geared towards an easy integration with the `no_std` Rust projects.
That said, it should be very easy to use in the `std` projects, too. A great emphasis
has been put on the clean implementation.

It comes with unit-tests tests, and it passes the
[6502 functional tests](https://github.com/Klaus2m5/6502_65C02_functional_tests).

This emulator is stricter than the real hardware and 6502 netlist-based
emulators such as [Perfect 6502](https://github.com/mist64/perfect6502). In short,
this is not a hardware simulator.

That said, there is no emulation of the microarch layer, e.g., no:

* cycle-accurate emulation or cycle counting,
* (unintended) support for the "undocumented" instructions. These instructions
  result in the execution jam, and the emulator will roll its state back to the
  previous instruction retruning an error on any subsequent invocation until
  it is reset.
* (unintended?) microarch side effects such as (not limited to):
    1. writing the old value first for the read-modify-write instructions,
    2. interrupt hijacking.

Stack underflows and overflows results in a fault, and the emulator will not
continue execution until it is reset.

## Examples

> To be done

## Other 6502-related resources and projects I have found inspiration in

### Emulators

1. [6502 functional tests](https://github.com/kromych/6502_65C02_functional_tests)
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
2. [CC65](https://github.com/cc65/cc65)
3. [LLVM-MOS](https://github.com/llvm-mos/llvm-mos)

### Documentation, reference

1. [Western Design Center](https://www.westerndesigncenter.com/)
2. [6502 Instruction Set](https://www.masswerk.at/6502/6502_instruction_set.html)
3. [6502.org](http://6502.org/)
