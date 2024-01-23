pub enum Exception {
    Interrupt           = 0,
    LoadAlignmentError  = 4,
    StoreAlignmentError = 5,
    Syscall             = 8,
    Break               = 9,
    IllegalInstruction  = 10,
    CoprocessorError    = 11,
    Overflow            = 12,
}

const EXCEPTION_JUMP_LOOKUP: [[u32; 4]; 2] = [
//   Reset        TLBMiss      Debug Break  General
    [0xbfc0_0000, 0x8000_0000, 0x8000_0040, 0x8000_0080], // BEV = 0
    [0xbfc0_0000, 0xbfc0_0100, 0xbfc0_0140, 0xbfc0_0180], // BEV = 1
];

pub enum ExceptionClass {
    Reset      = 0,
    TLBMiss    = 1,    // Only applies to user region KUSEG
    DebugBreak = 2,
    General    = 3,
}

pub enum ExceptionVector {
    Normal = 0,
    Boot   = 1,
}

pub fn handler(vector: ExceptionVector, class: ExceptionClass) -> u32 {
    EXCEPTION_JUMP_LOOKUP[vector as usize][class as usize]
}