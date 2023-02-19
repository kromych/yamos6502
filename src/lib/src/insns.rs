///! Instruction Data

/// String representation for instructions.
///
/// Each 8-bit opcode below is split into 6 bits that
/// represent Operation and Adressing Mode, and 2 bits that
/// indicate the instruction group.
const INSN_STR: [&'static str; 256] = [
    // Mnemonic     Opcode
    "BRK",       // 0x00    0b000000    0b00
    "ORA X,ind", // 0x01    0b000000    0b01
    "JAM",       // 0x02    0b000000    0b10
    "JAM",       // 0x03    0b000000    0b11
    "JAM",       // 0x04    0b000001    0b00
    "ORA zpg",   // 0x05    0b000001    0b01
    "ASL zpg",   // 0x06    0b000001    0b10
    "JAM",       // 0x07    0b000001    0b11
    "PHP",       // 0x08    0b000010    0b00
    "ORA imm",   // 0x09    0b000010    0b01
    "ASL",       // 0x0a    0b000010    0b10
    "JAM",       // 0x0b    0b000010    0b11
    "JAM",       // 0x0c    0b000011    0b00
    "ORA abs",   // 0x0d    0b000011    0b01
    "ASL abs",   // 0x0e    0b000011    0b10
    "JAM",       // 0x0f    0b000011    0b11
    "BPL rel",   // 0x10    0b000100    0b00
    "ORA ind,Y", // 0x11    0b000100    0b01
    "JAM",       // 0x12    0b000100    0b10
    "JAM",       // 0x13    0b000100    0b11
    "JAM",       // 0x14    0b000101    0b00
    "ORA zpg,X", // 0x15    0b000101    0b01
    "ASL zpg,X", // 0x16    0b000101    0b10
    "JAM",       // 0x17    0b000101    0b11
    "CLC",       // 0x18    0b000110    0b00
    "ORA abs,Y", // 0x19    0b000110    0b01
    "JAM",       // 0x1a    0b000110    0b10
    "JAM",       // 0x1b    0b000110    0b11
    "JAM",       // 0x1c    0b000111    0b00
    "ORA abs,X", // 0x1d    0b000111    0b01
    "ASL abs,X", // 0x1e    0b000111    0b10
    "JAM",       // 0x1f    0b000111    0b11
    "JSR abs",   // 0x20    0b001000    0b00
    "AND X,ind", // 0x21    0b001000    0b01
    "JAM",       // 0x22    0b001000    0b10
    "JAM",       // 0x23    0b001000    0b11
    "BIT zpg",   // 0x24    0b001001    0b00
    "AND zpg",   // 0x25    0b001001    0b01
    "ROL zpg",   // 0x26    0b001001    0b10
    "JAM",       // 0x27    0b001001    0b11
    "PLP",       // 0x28    0b001010    0b00
    "AND imm",   // 0x29    0b001010    0b01
    "ROL",       // 0x2a    0b001010    0b10
    "JAM",       // 0x2b    0b001010    0b11
    "BIT abs",   // 0x2c    0b001011    0b00
    "AND abs",   // 0x2d    0b001011    0b01
    "ROL abs",   // 0x2e    0b001011    0b10
    "JAM",       // 0x2f    0b001011    0b11
    "BMI rel",   // 0x30    0b001100    0b00
    "AND ind,Y", // 0x31    0b001100    0b01
    "JAM",       // 0x32    0b001100    0b10
    "JAM",       // 0x33    0b001100    0b11
    "JAM",       // 0x34    0b001101    0b00
    "AND zpg,X", // 0x35    0b001101    0b01
    "ROL zpg,X", // 0x36    0b001101    0b10
    "JAM",       // 0x37    0b001101    0b11
    "SEC",       // 0x38    0b001110    0b00
    "AND abs,Y", // 0x39    0b001110    0b01
    "JAM",       // 0x3a    0b001110    0b10
    "JAM",       // 0x3b    0b001110    0b11
    "JAM",       // 0x3c    0b001111    0b00
    "AND abs,X", // 0x3d    0b001111    0b01
    "ROL abs,X", // 0x3e    0b001111    0b10
    "JAM",       // 0x3f    0b001111    0b11
    "RTI",       // 0x40    0b010000    0b00
    "EOR X,ind", // 0x41    0b010000    0b01
    "JAM",       // 0x42    0b010000    0b10
    "JAM",       // 0x43    0b010000    0b11
    "JAM",       // 0x44    0b010001    0b00
    "EOR zpg",   // 0x45    0b010001    0b01
    "LSR zpg",   // 0x46    0b010001    0b10
    "JAM",       // 0x47    0b010001    0b11
    "PHA",       // 0x48    0b010010    0b00
    "EOR imm",   // 0x49    0b010010    0b01
    "LSR",       // 0x4a    0b010010    0b10
    "JAM",       // 0x4b    0b010010    0b11
    "JMP abs",   // 0x4c    0b010011    0b00
    "EOR abs",   // 0x4d    0b010011    0b01
    "LSR abs",   // 0x4e    0b010011    0b10
    "JAM",       // 0x4f    0b010011    0b11
    "BVC rel",   // 0x50    0b010100    0b00
    "EOR ind,Y", // 0x51    0b010100    0b01
    "JAM",       // 0x52    0b010100    0b10
    "JAM",       // 0x53    0b010100    0b11
    "JAM",       // 0x54    0b010101    0b00
    "EOR zpg,X", // 0x55    0b010101    0b01
    "LSR zpg,X", // 0x56    0b010101    0b10
    "JAM",       // 0x57    0b010101    0b11
    "CLI",       // 0x58    0b010110    0b00
    "EOR abs,Y", // 0x59    0b010110    0b01
    "JAM",       // 0x5a    0b010110    0b10
    "JAM",       // 0x5b    0b010110    0b11
    "JAM",       // 0x5c    0b010111    0b00
    "EOR abs,X", // 0x5d    0b010111    0b01
    "LSR abs,X", // 0x5e    0b010111    0b10
    "JAM",       // 0x5f    0b010111    0b11
    "RTS",       // 0x60    0b011000    0b00
    "ADC X,ind", // 0x61    0b011000    0b01
    "JAM",       // 0x62    0b011000    0b10
    "JAM",       // 0x63    0b011000    0b11
    "JAM",       // 0x64    0b011001    0b00
    "ADC zpg",   // 0x65    0b011001    0b01
    "ROR zpg",   // 0x66    0b011001    0b10
    "JAM",       // 0x67    0b011001    0b11
    "PLA",       // 0x68    0b011010    0b00
    "ADC imm",   // 0x69    0b011010    0b01
    "ROR",       // 0x6a    0b011010    0b10
    "JAM",       // 0x6b    0b011010    0b11
    "JMP ind",   // 0x6c    0b011011    0b00
    "ADC abs",   // 0x6d    0b011011    0b01
    "ROR abs",   // 0x6e    0b011011    0b10
    "JAM",       // 0x6f    0b011011    0b11
    "BVS rel",   // 0x70    0b011100    0b00
    "ADC ind,Y", // 0x71    0b011100    0b01
    "JAM",       // 0x72    0b011100    0b10
    "JAM",       // 0x73    0b011100    0b11
    "JAM",       // 0x74    0b011101    0b00
    "ADC zpg,X", // 0x75    0b011101    0b01
    "ROR zpg,X", // 0x76    0b011101    0b10
    "JAM",       // 0x77    0b011101    0b11
    "SEI",       // 0x78    0b011110    0b00
    "ADC abs,Y", // 0x79    0b011110    0b01
    "JAM",       // 0x7a    0b011110    0b10
    "JAM",       // 0x7b    0b011110    0b11
    "JAM",       // 0x7c    0b011111    0b00
    "ADC abs,X", // 0x7d    0b011111    0b01
    "ROR abs,X", // 0x7e    0b011111    0b10
    "JAM",       // 0x7f    0b011111    0b11
    "JAM",       // 0x80    0b100000    0b00
    "STA X,ind", // 0x81    0b100000    0b01
    "JAM",       // 0x82    0b100000    0b10
    "JAM",       // 0x83    0b100000    0b11
    "STY zpg",   // 0x84    0b100001    0b00
    "STA zpg",   // 0x85    0b100001    0b01
    "STX zpg",   // 0x86    0b100001    0b10
    "JAM",       // 0x87    0b100001    0b11
    "DEY",       // 0x88    0b100010    0b00
    "JAM",       // 0x89    0b100010    0b01
    "TXA",       // 0x8a    0b100010    0b10
    "JAM",       // 0x8b    0b100010    0b11
    "STY abs",   // 0x8c    0b100011    0b00
    "STA abs",   // 0x8d    0b100011    0b01
    "STX abs",   // 0x8e    0b100011    0b10
    "JAM",       // 0x8f    0b100011    0b11
    "BCC rel",   // 0x90    0b100100    0b00
    "STA ind,Y", // 0x91    0b100100    0b01
    "JAM",       // 0x92    0b100100    0b10
    "JAM",       // 0x93    0b100100    0b11
    "STY zpg,X", // 0x94    0b100101    0b00
    "STA zpg,X", // 0x95    0b100101    0b01
    "STX zpg,Y", // 0x96    0b100101    0b10
    "JAM",       // 0x97    0b100101    0b11
    "TYA",       // 0x98    0b100110    0b00
    "STA abs,Y", // 0x99    0b100110    0b01
    "TXS",       // 0x9a    0b100110    0b10
    "JAM",       // 0x9b    0b100110    0b11
    "JAM",       // 0x9c    0b100111    0b00
    "STA abs,X", // 0x9d    0b100111    0b01
    "JAM",       // 0x9e    0b100111    0b10
    "JAM",       // 0x9f    0b100111    0b11
    "LDY imm",   // 0xa0    0b101000    0b00
    "LDA X,ind", // 0xa1    0b101000    0b01
    "LDX imm",   // 0xa2    0b101000    0b10
    "JAM",       // 0xa3    0b101000    0b11
    "LDY zpg",   // 0xa4    0b101001    0b00
    "LDA zpg",   // 0xa5    0b101001    0b01
    "LDX zpg",   // 0xa6    0b101001    0b10
    "JAM",       // 0xa7    0b101001    0b11
    "TAY",       // 0xa8    0b101010    0b00
    "LDA imm",   // 0xa9    0b101010    0b01
    "TAX",       // 0xaa    0b101010    0b10
    "JAM",       // 0xab    0b101010    0b11
    "LDY abs",   // 0xac    0b101011    0b00
    "LDA abs",   // 0xad    0b101011    0b01
    "LDX abs",   // 0xae    0b101011    0b10
    "JAM",       // 0xaf    0b101011    0b11
    "BCS rel",   // 0xb0    0b101100    0b00
    "LDA ind,Y", // 0xb1    0b101100    0b01
    "JAM",       // 0xb2    0b101100    0b10
    "JAM",       // 0xb3    0b101100    0b11
    "LDY zpg,X", // 0xb4    0b101101    0b00
    "LDA zpg,X", // 0xb5    0b101101    0b01
    "LDX zpg,Y", // 0xb6    0b101101    0b10
    "JAM",       // 0xb7    0b101101    0b11
    "CLV",       // 0xb8    0b101110    0b00
    "LDA abs,Y", // 0xb9    0b101110    0b01
    "TSX",       // 0xba    0b101110    0b10
    "JAM",       // 0xbb    0b101110    0b11
    "LDY abs,X", // 0xbc    0b101111    0b00
    "LDA abs,X", // 0xbd    0b101111    0b01
    "LDX abs,Y", // 0xbe    0b101111    0b10
    "JAM",       // 0xbf    0b101111    0b11
    "CPY imm",   // 0xc0    0b110000    0b00
    "CMP X,ind", // 0xc1    0b110000    0b01
    "JAM",       // 0xc2    0b110000    0b10
    "JAM",       // 0xc3    0b110000    0b11
    "CPY zpg",   // 0xc4    0b110001    0b00
    "CMP zpg",   // 0xc5    0b110001    0b01
    "DEC zpg",   // 0xc6    0b110001    0b10
    "JAM",       // 0xc7    0b110001    0b11
    "INY",       // 0xc8    0b110010    0b00
    "CMP imm",   // 0xc9    0b110010    0b01
    "DEX",       // 0xca    0b110010    0b10
    "JAM",       // 0xcb    0b110010    0b11
    "CPY abs",   // 0xcc    0b110011    0b00
    "CMP abs",   // 0xcd    0b110011    0b01
    "DEC abs",   // 0xce    0b110011    0b10
    "JAM",       // 0xcf    0b110011    0b11
    "BNE rel",   // 0xd0    0b110100    0b00
    "CMP ind,Y", // 0xd1    0b110100    0b01
    "JAM",       // 0xd2    0b110100    0b10
    "JAM",       // 0xd3    0b110100    0b11
    "JAM",       // 0xd4    0b110101    0b00
    "CMP zpg,X", // 0xd5    0b110101    0b01
    "DEC zpg,X", // 0xd6    0b110101    0b10
    "JAM",       // 0xd7    0b110101    0b11
    "CLD",       // 0xd8    0b110110    0b00
    "CMP abs,Y", // 0xd9    0b110110    0b01
    "JAM",       // 0xda    0b110110    0b10
    "JAM",       // 0xdb    0b110110    0b11
    "JAM",       // 0xdc    0b110111    0b00
    "CMP abs,X", // 0xdd    0b110111    0b01
    "DEC abs,X", // 0xde    0b110111    0b10
    "JAM",       // 0xdf    0b110111    0b11
    "CPX imm",   // 0xe0    0b111000    0b00
    "SBC X,ind", // 0xe1    0b111000    0b01
    "JAM",       // 0xe2    0b111000    0b10
    "JAM",       // 0xe3    0b111000    0b11
    "CPX zpg",   // 0xe4    0b111001    0b00
    "SBC zpg",   // 0xe5    0b111001    0b01
    "INC zpg",   // 0xe6    0b111001    0b10
    "JAM",       // 0xe7    0b111001    0b11
    "INX",       // 0xe8    0b111010    0b00
    "SBC imm",   // 0xe9    0b111010    0b01
    "NOP",       // 0xea    0b111010    0b10
    "JAM",       // 0xeb    0b111010    0b11
    "CPX abs",   // 0xec    0b111011    0b00
    "SBC abs",   // 0xed    0b111011    0b01
    "INC abs",   // 0xee    0b111011    0b10
    "JAM",       // 0xef    0b111011    0b11
    "BEQ rel",   // 0xf0    0b111100    0b00
    "SBC ind,Y", // 0xf1    0b111100    0b01
    "JAM",       // 0xf2    0b111100    0b10
    "JAM",       // 0xf3    0b111100    0b11
    "JAM",       // 0xf4    0b111101    0b00
    "SBC zpg,X", // 0xf5    0b111101    0b01
    "INC zpg,X", // 0xf6    0b111101    0b10
    "JAM",       // 0xf7    0b111101    0b11
    "SED",       // 0xf8    0b111110    0b00
    "SBC abs,Y", // 0xf9    0b111110    0b01
    "JAM",       // 0xfa    0b111110    0b10
    "JAM",       // 0xfb    0b111110    0b11
    "JAM",       // 0xfc    0b111111    0b00
    "SBC abs,X", // 0xfd    0b111111    0b01
    "INC abs,X", // 0xfe    0b111111    0b10
    "JAM",       // 0xff    0b111111    0b11
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    Immediate,
    Relative,
    Xindirect,
    Indirect,
    IndirectY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Zeropage,
    ZeropageX,
    ZeropageY,
}

/// Instruction representation.
/// Nor particulary space-efficient (perhaps makes the code slower, too?),
/// yet readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insn {
    ADC(AddressMode),
    AND(AddressMode),
    ASLA,
    ASL(AddressMode),
    BCC(AddressMode),
    BCS(AddressMode),
    BEQ(AddressMode),
    BIT(AddressMode),
    BMI(AddressMode),
    BNE(AddressMode),
    BPL(AddressMode),
    BRK,
    BVC(AddressMode),
    BVS(AddressMode),
    CLC,
    CLD,
    CLI,
    CLV,
    CMP(AddressMode),
    CPX(AddressMode),
    CPY(AddressMode),
    DEC(AddressMode),
    DEX,
    DEY,
    EOR(AddressMode),
    INC(AddressMode),
    INX,
    INY,
    JAM,
    JMP(AddressMode),
    JSR(AddressMode),
    LDA(AddressMode),
    LDX(AddressMode),
    LDY(AddressMode),
    LSRA,
    LSR(AddressMode),
    NOP,
    ORA(AddressMode),
    PHA,
    PHP,
    PLA,
    PLP,
    ROLA,
    ROL(AddressMode),
    RORA,
    ROR(AddressMode),
    RTI,
    RTS,
    SBC(AddressMode),
    SEC,
    SED,
    SEI,
    STA(AddressMode),
    STX(AddressMode),
    STY(AddressMode),
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

impl Insn {
    pub fn is_valid(&self) -> bool {
        *self != Insn::JAM
    }
}

const INSN_BY_GROUP: [[[Insn; 8]; 8]; 4] = [
    // Group 0b00. Flags, conditionals, jumps, misc. There are a few
    // quite complex instructions here.
    [
        [
            Insn::BRK,
            Insn::JAM,
            Insn::PHP,
            Insn::JAM,
            Insn::BPL(AddressMode::Relative),
            Insn::JAM,
            Insn::CLC,
            Insn::JAM,
        ],
        [
            Insn::JSR(AddressMode::Absolute),
            Insn::BIT(AddressMode::Zeropage),
            Insn::PLP,
            Insn::BIT(AddressMode::Absolute),
            Insn::BMI(AddressMode::Relative),
            Insn::JAM,
            Insn::SEC,
            Insn::JAM,
        ],
        [
            Insn::RTI,
            Insn::JAM,
            Insn::PHA,
            Insn::JMP(AddressMode::Absolute),
            Insn::BVC(AddressMode::Relative),
            Insn::JAM,
            Insn::CLI,
            Insn::JAM,
        ],
        [
            Insn::RTS,
            Insn::JAM,
            Insn::PLA,
            Insn::JMP(AddressMode::Indirect),
            Insn::BVS(AddressMode::Relative),
            Insn::JAM,
            Insn::SEI,
            Insn::JAM,
        ],
        [
            Insn::JAM,
            Insn::STY(AddressMode::Zeropage),
            Insn::DEY,
            Insn::STY(AddressMode::Absolute),
            Insn::BCC(AddressMode::Relative),
            Insn::STY(AddressMode::ZeropageX),
            Insn::TYA,
            Insn::JAM,
        ],
        [
            Insn::LDY(AddressMode::Immediate),
            Insn::LDY(AddressMode::Zeropage),
            Insn::TAY,
            Insn::LDY(AddressMode::Absolute),
            Insn::BCS(AddressMode::Relative),
            Insn::LDY(AddressMode::ZeropageX),
            Insn::CLV,
            Insn::LDY(AddressMode::AbsoluteX),
        ],
        [
            Insn::CPY(AddressMode::Immediate),
            Insn::CPY(AddressMode::Zeropage),
            Insn::INY,
            Insn::CPY(AddressMode::Absolute),
            Insn::BNE(AddressMode::Relative),
            Insn::JAM,
            Insn::CLD,
            Insn::JAM,
        ],
        [
            Insn::CPX(AddressMode::Immediate),
            Insn::CPX(AddressMode::Zeropage),
            Insn::INX,
            Insn::CPX(AddressMode::Absolute),
            Insn::BEQ(AddressMode::Relative),
            Insn::JAM,
            Insn::SED,
            Insn::JAM,
        ],
    ],
    // Group 0b01. ALU instructions, very regular encoding
    // to make decoding and execution faster in hardware.
    [
        [
            Insn::ORA(AddressMode::Xindirect),
            Insn::ORA(AddressMode::Zeropage),
            Insn::ORA(AddressMode::Immediate),
            Insn::ORA(AddressMode::Absolute),
            Insn::ORA(AddressMode::IndirectY),
            Insn::ORA(AddressMode::ZeropageX),
            Insn::ORA(AddressMode::AbsoluteY),
            Insn::ORA(AddressMode::AbsoluteX),
        ],
        [
            Insn::AND(AddressMode::Xindirect),
            Insn::AND(AddressMode::Zeropage),
            Insn::AND(AddressMode::Immediate),
            Insn::AND(AddressMode::Absolute),
            Insn::AND(AddressMode::IndirectY),
            Insn::AND(AddressMode::ZeropageX),
            Insn::AND(AddressMode::AbsoluteY),
            Insn::AND(AddressMode::AbsoluteX),
        ],
        [
            Insn::EOR(AddressMode::Xindirect),
            Insn::EOR(AddressMode::Zeropage),
            Insn::EOR(AddressMode::Immediate),
            Insn::EOR(AddressMode::Absolute),
            Insn::EOR(AddressMode::IndirectY),
            Insn::EOR(AddressMode::ZeropageX),
            Insn::EOR(AddressMode::AbsoluteY),
            Insn::EOR(AddressMode::AbsoluteX),
        ],
        [
            Insn::ADC(AddressMode::Xindirect),
            Insn::ADC(AddressMode::Zeropage),
            Insn::ADC(AddressMode::Immediate),
            Insn::ADC(AddressMode::Absolute),
            Insn::ADC(AddressMode::IndirectY),
            Insn::ADC(AddressMode::ZeropageX),
            Insn::ADC(AddressMode::AbsoluteY),
            Insn::ADC(AddressMode::AbsoluteX),
        ],
        [
            Insn::STA(AddressMode::Xindirect),
            Insn::STA(AddressMode::Zeropage),
            Insn::JAM,
            Insn::STA(AddressMode::Absolute),
            Insn::STA(AddressMode::IndirectY),
            Insn::STA(AddressMode::ZeropageX),
            Insn::STA(AddressMode::AbsoluteY),
            Insn::STA(AddressMode::AbsoluteX),
        ],
        [
            Insn::LDA(AddressMode::Xindirect),
            Insn::LDA(AddressMode::Zeropage),
            Insn::LDA(AddressMode::Immediate),
            Insn::LDA(AddressMode::Absolute),
            Insn::LDA(AddressMode::IndirectY),
            Insn::LDA(AddressMode::ZeropageX),
            Insn::LDA(AddressMode::AbsoluteY),
            Insn::LDA(AddressMode::AbsoluteX),
        ],
        [
            Insn::CMP(AddressMode::Xindirect),
            Insn::CMP(AddressMode::Zeropage),
            Insn::CMP(AddressMode::Immediate),
            Insn::CMP(AddressMode::Absolute),
            Insn::CMP(AddressMode::IndirectY),
            Insn::CMP(AddressMode::ZeropageX),
            Insn::CMP(AddressMode::AbsoluteY),
            Insn::CMP(AddressMode::AbsoluteX),
        ],
        [
            Insn::SBC(AddressMode::Xindirect),
            Insn::SBC(AddressMode::Zeropage),
            Insn::SBC(AddressMode::Immediate),
            Insn::SBC(AddressMode::Absolute),
            Insn::SBC(AddressMode::IndirectY),
            Insn::SBC(AddressMode::ZeropageX),
            Insn::SBC(AddressMode::AbsoluteY),
            Insn::SBC(AddressMode::AbsoluteX),
        ],
    ],
    // Group 0b10. Bit operation and accumulator operations,
    // less regular than the ALU group.
    [
        [
            Insn::JAM,
            Insn::ASL(AddressMode::Zeropage),
            Insn::ASLA,
            Insn::ASL(AddressMode::Absolute),
            Insn::JAM,
            Insn::ASL(AddressMode::ZeropageX),
            Insn::JAM,
            Insn::ASL(AddressMode::AbsoluteX),
        ],
        [
            Insn::JAM,
            Insn::ROL(AddressMode::Zeropage),
            Insn::ROLA,
            Insn::ROL(AddressMode::Absolute),
            Insn::JAM,
            Insn::ROL(AddressMode::ZeropageX),
            Insn::JAM,
            Insn::ROL(AddressMode::AbsoluteX),
        ],
        [
            Insn::JAM,
            Insn::LSR(AddressMode::Zeropage),
            Insn::LSRA,
            Insn::LSR(AddressMode::Absolute),
            Insn::JAM,
            Insn::LSR(AddressMode::ZeropageX),
            Insn::JAM,
            Insn::LSR(AddressMode::AbsoluteX),
        ],
        [
            Insn::JAM,
            Insn::ROR(AddressMode::Zeropage),
            Insn::RORA,
            Insn::ROR(AddressMode::Absolute),
            Insn::JAM,
            Insn::ROR(AddressMode::ZeropageX),
            Insn::JAM,
            Insn::ROR(AddressMode::AbsoluteX),
        ],
        [
            Insn::JAM,
            Insn::STX(AddressMode::Zeropage),
            Insn::TXA,
            Insn::STX(AddressMode::Absolute),
            Insn::JAM,
            Insn::STX(AddressMode::ZeropageY),
            Insn::TXS,
            Insn::JAM,
        ],
        [
            Insn::LDX(AddressMode::Immediate),
            Insn::LDX(AddressMode::Zeropage),
            Insn::TAX,
            Insn::LDX(AddressMode::Absolute),
            Insn::JAM,
            Insn::LDX(AddressMode::ZeropageY),
            Insn::TSX,
            Insn::LDX(AddressMode::AbsoluteY),
        ],
        [
            Insn::JAM,
            Insn::DEC(AddressMode::Zeropage),
            Insn::DEX,
            Insn::DEC(AddressMode::Absolute),
            Insn::JAM,
            Insn::DEC(AddressMode::ZeropageX),
            Insn::JAM,
            Insn::DEC(AddressMode::AbsoluteX),
        ],
        [
            Insn::JAM,
            Insn::INC(AddressMode::Zeropage),
            Insn::NOP,
            Insn::INC(AddressMode::Absolute),
            Insn::JAM,
            Insn::INC(AddressMode::ZeropageX),
            Insn::JAM,
            Insn::INC(AddressMode::AbsoluteX),
        ],
    ],
    // Group 0b11. There are no valid instructions in this group.
    [
        [
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
        ],
        [
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
        ],
        [
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
        ],
        [
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
        ],
        [
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
        ],
        [
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
        ],
        [
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
        ],
        [
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
            Insn::JAM,
        ],
    ],
];

pub fn decode_insn(opcode: u8) -> Insn {
    let group = (opcode & 0b11) as usize;
    let two_octals = (opcode >> 2) as usize;
    let lo_octal = two_octals & 0b111;
    let hi_octal = two_octals >> 3;

    INSN_BY_GROUP[group][hi_octal][lo_octal]
}

pub fn encode_insn(insn: Insn) -> u8 {
    match insn {
        Insn::ADC(AddressMode::Absolute) => 0x6d,
        Insn::ADC(AddressMode::AbsoluteX) => 0x7d,
        Insn::ADC(AddressMode::AbsoluteY) => 0x79,
        Insn::ADC(AddressMode::Immediate) => 0x69,
        Insn::ADC(AddressMode::IndirectY) => 0x71,
        Insn::ADC(AddressMode::Xindirect) => 0x61,
        Insn::ADC(AddressMode::Zeropage) => 0x65,
        Insn::ADC(AddressMode::ZeropageX) => 0x75,
        Insn::AND(AddressMode::Absolute) => 0x2d,
        Insn::AND(AddressMode::AbsoluteX) => 0x3d,
        Insn::AND(AddressMode::AbsoluteY) => 0x39,
        Insn::AND(AddressMode::Immediate) => 0x29,
        Insn::AND(AddressMode::IndirectY) => 0x31,
        Insn::AND(AddressMode::Xindirect) => 0x21,
        Insn::AND(AddressMode::Zeropage) => 0x25,
        Insn::AND(AddressMode::ZeropageX) => 0x35,
        Insn::ASL(AddressMode::Absolute) => 0x0e,
        Insn::ASL(AddressMode::AbsoluteX) => 0x1e,
        Insn::ASLA => 0x0a,
        Insn::ASL(AddressMode::Zeropage) => 0x06,
        Insn::ASL(AddressMode::ZeropageX) => 0x16,
        Insn::BCC(AddressMode::Relative) => 0x90,
        Insn::BCS(AddressMode::Relative) => 0xb0,
        Insn::BEQ(AddressMode::Relative) => 0xf0,
        Insn::BIT(AddressMode::Absolute) => 0x2c,
        Insn::BIT(AddressMode::Zeropage) => 0x24,
        Insn::BMI(AddressMode::Relative) => 0x30,
        Insn::BNE(AddressMode::Relative) => 0xd0,
        Insn::BPL(AddressMode::Relative) => 0x10,
        Insn::BRK => 0x00,
        Insn::BVC(AddressMode::Relative) => 0x50,
        Insn::BVS(AddressMode::Relative) => 0x70,
        Insn::CLC => 0x18,
        Insn::CLD => 0xd8,
        Insn::CLI => 0x58,
        Insn::CLV => 0xb8,
        Insn::CMP(AddressMode::Absolute) => 0xcd,
        Insn::CMP(AddressMode::AbsoluteX) => 0xdd,
        Insn::CMP(AddressMode::AbsoluteY) => 0xd9,
        Insn::CMP(AddressMode::Immediate) => 0xc9,
        Insn::CMP(AddressMode::IndirectY) => 0xd1,
        Insn::CMP(AddressMode::Xindirect) => 0xc1,
        Insn::CMP(AddressMode::Zeropage) => 0xc5,
        Insn::CMP(AddressMode::ZeropageX) => 0xd5,
        Insn::CPX(AddressMode::Absolute) => 0xec,
        Insn::CPX(AddressMode::Immediate) => 0xe0,
        Insn::CPX(AddressMode::Zeropage) => 0xe4,
        Insn::CPY(AddressMode::Absolute) => 0xcc,
        Insn::CPY(AddressMode::Immediate) => 0xc0,
        Insn::CPY(AddressMode::Zeropage) => 0xc4,
        Insn::DEC(AddressMode::Absolute) => 0xce,
        Insn::DEC(AddressMode::AbsoluteX) => 0xde,
        Insn::DEC(AddressMode::Zeropage) => 0xc6,
        Insn::DEC(AddressMode::ZeropageX) => 0xd6,
        Insn::DEX => 0xca,
        Insn::DEY => 0x88,
        Insn::EOR(AddressMode::Absolute) => 0x4d,
        Insn::EOR(AddressMode::AbsoluteX) => 0x5d,
        Insn::EOR(AddressMode::AbsoluteY) => 0x59,
        Insn::EOR(AddressMode::Immediate) => 0x49,
        Insn::EOR(AddressMode::IndirectY) => 0x51,
        Insn::EOR(AddressMode::Xindirect) => 0x41,
        Insn::EOR(AddressMode::Zeropage) => 0x45,
        Insn::EOR(AddressMode::ZeropageX) => 0x55,
        Insn::INC(AddressMode::Absolute) => 0xee,
        Insn::INC(AddressMode::AbsoluteX) => 0xfe,
        Insn::INC(AddressMode::Zeropage) => 0xe6,
        Insn::INC(AddressMode::ZeropageX) => 0xf6,
        Insn::INX => 0xe8,
        Insn::INY => 0xc8,
        Insn::JMP(AddressMode::Absolute) => 0x4c,
        Insn::JMP(AddressMode::Indirect) => 0x6c,
        Insn::JSR(AddressMode::Absolute) => 0x20,
        Insn::LDA(AddressMode::Absolute) => 0xad, // got a test
        Insn::LDA(AddressMode::AbsoluteX) => 0xbd, // got a test
        Insn::LDA(AddressMode::AbsoluteY) => 0xb9, // got a test
        Insn::LDA(AddressMode::Immediate) => 0xa9, // got a test
        Insn::LDA(AddressMode::IndirectY) => 0xb1, // got a test
        Insn::LDA(AddressMode::Xindirect) => 0xa1, // got a test
        Insn::LDA(AddressMode::Zeropage) => 0xa5, // got a test
        Insn::LDA(AddressMode::ZeropageX) => 0xb5, // got a test
        Insn::LDX(AddressMode::Absolute) => 0xae, // got a test
        Insn::LDX(AddressMode::AbsoluteY) => 0xbe, // got a test
        Insn::LDX(AddressMode::Immediate) => 0xa2, // got a test
        Insn::LDX(AddressMode::Zeropage) => 0xa6, // got a test
        Insn::LDX(AddressMode::ZeropageY) => 0xb6, // got a test
        Insn::LDY(AddressMode::Absolute) => 0xac, // got a test
        Insn::LDY(AddressMode::AbsoluteX) => 0xbc, // got a test
        Insn::LDY(AddressMode::Immediate) => 0xa0, // got a test
        Insn::LDY(AddressMode::Zeropage) => 0xa4, // got a test
        Insn::LDY(AddressMode::ZeropageX) => 0xb4, // got a test
        Insn::LSR(AddressMode::Absolute) => 0x4e,
        Insn::LSR(AddressMode::AbsoluteX) => 0x5e,
        Insn::LSRA => 0x4a,
        Insn::LSR(AddressMode::Zeropage) => 0x46,
        Insn::LSR(AddressMode::ZeropageX) => 0x56,
        Insn::NOP => 0xea,
        Insn::ORA(AddressMode::Absolute) => 0x0d,
        Insn::ORA(AddressMode::AbsoluteX) => 0x1d,
        Insn::ORA(AddressMode::AbsoluteY) => 0x19,
        Insn::ORA(AddressMode::Immediate) => 0x09,
        Insn::ORA(AddressMode::IndirectY) => 0x11,
        Insn::ORA(AddressMode::Xindirect) => 0x01,
        Insn::ORA(AddressMode::Zeropage) => 0x05,
        Insn::ORA(AddressMode::ZeropageX) => 0x15,
        Insn::PHA => 0x48,
        Insn::PHP => 0x08,
        Insn::PLA => 0x68,
        Insn::PLP => 0x28,
        Insn::ROL(AddressMode::Absolute) => 0x2e,
        Insn::ROL(AddressMode::AbsoluteX) => 0x3e,
        Insn::ROLA => 0x2a,
        Insn::ROL(AddressMode::Zeropage) => 0x26,
        Insn::ROL(AddressMode::ZeropageX) => 0x36,
        Insn::ROR(AddressMode::Absolute) => 0x6e,
        Insn::ROR(AddressMode::AbsoluteX) => 0x7e,
        Insn::RORA => 0x6a,
        Insn::ROR(AddressMode::Zeropage) => 0x66,
        Insn::ROR(AddressMode::ZeropageX) => 0x76,
        Insn::RTI => 0x40,
        Insn::RTS => 0x60,
        Insn::SBC(AddressMode::Absolute) => 0xed,
        Insn::SBC(AddressMode::AbsoluteX) => 0xfd,
        Insn::SBC(AddressMode::AbsoluteY) => 0xf9,
        Insn::SBC(AddressMode::Immediate) => 0xe9,
        Insn::SBC(AddressMode::IndirectY) => 0xf1,
        Insn::SBC(AddressMode::Xindirect) => 0xe1,
        Insn::SBC(AddressMode::Zeropage) => 0xe5,
        Insn::SBC(AddressMode::ZeropageX) => 0xf5,
        Insn::SEC => 0x38,
        Insn::SED => 0xf8,
        Insn::SEI => 0x78,
        Insn::STA(AddressMode::Absolute) => 0x8d,
        Insn::STA(AddressMode::AbsoluteX) => 0x9d,
        Insn::STA(AddressMode::AbsoluteY) => 0x99,
        Insn::STA(AddressMode::IndirectY) => 0x91,
        Insn::STA(AddressMode::Xindirect) => 0x81,
        Insn::STA(AddressMode::Zeropage) => 0x85,
        Insn::STA(AddressMode::ZeropageX) => 0x95,
        Insn::STX(AddressMode::Absolute) => 0x8e,
        Insn::STX(AddressMode::Zeropage) => 0x86,
        Insn::STX(AddressMode::ZeropageY) => 0x96,
        Insn::STY(AddressMode::Absolute) => 0x8c,
        Insn::STY(AddressMode::Zeropage) => 0x84,
        Insn::STY(AddressMode::ZeropageX) => 0x94,
        Insn::TAX => 0xaa,
        Insn::TAY => 0xa8,
        Insn::TSX => 0xba,
        Insn::TXA => 0x8a,
        Insn::TXS => 0x9a,
        Insn::TYA => 0x98,
        _ => 0xff,
    }
}

pub fn get_opcode_string(opcode: u8) -> &'static str {
    INSN_STR[opcode as usize]
}
