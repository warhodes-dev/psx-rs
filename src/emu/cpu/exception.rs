pub enum Exception {
    Interrupt = 0,
    LoadAlignmentError = 4,
    StoreAlignmentError = 5,
    Syscall   = 8,
    Overflow  = 12,
}

const EXCEPTION_VECTORS: [[u32; 4]; 2] = [
//   Reset        TLBMiss      Debug Break  General
    [0xbfc0_0000, 0x8000_0000, 0x8000_0040, 0x8000_0080], // BEV = 0
    [0xbfc0_0000, 0xbfc0_0100, 0xbfc0_0140, 0xbfc0_0180], // BEV = 1
];

pub enum ExceptionClass {
    Reset,
    TLBMiss,    // Only applies to user region KUSEG
    DebugBreak,
    General,
}

pub enum ExceptionVector {
    Normal = 0,
    Boot = 1,
}

pub fn handler(vector: ExceptionVector, class: ExceptionClass) -> u32 {
    EXCEPTION_VECTORS[vector as usize][class as usize]

    /*
    use ExceptionVector::{Boot, Normal};
    use ExceptionClass::{Reset, TLBMiss, DebugBreak, General};

    match (vector, class) {
        (_, Reset)           => 0xbfc0_0000,

        (Boot, TLBMiss)      => 0xbfc0_0100,
        (Boot, DebugBreak)   => 0xbfc0_0140,
        (Boot, General)      => 0xbfc0_0180,

        (Normal, TLBMiss)    => 0x8000_0000,
        (Normal, DebugBreak) => 0x8000_0040,
        (Normal, General)    => 0x8000_0080,
    }
    */
}

