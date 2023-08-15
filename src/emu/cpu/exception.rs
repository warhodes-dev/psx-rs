pub enum Exception {
    Syscall = 0x8,
}

const EXCEPTION_VECTOR: [[u32; 4]; 2] = [
//   Reset        Vmem (N/A)   Debug Break  General
    [0xbfc0_0000, 0x8000_0000, 0x8000_0040, 0x8000_0080], // BEV = 0
    [0xbfc0_0000, 0xbfc0_0100, 0xbfc0_0140, 0xbfc0_0180]  // BEV = 1
];

pub enum ExceptionClass {
    Reset = 0,
    Vmem = 1,
    DebugBreak = 2,
    General = 3,
}

pub enum BootExceptionVector {
    BEV0 = 0,
    BEV1 = 1,
}

pub fn handler(bev: BootExceptionVector, class: ExceptionClass) -> u32 {
    EXCEPTION_VECTOR[bev as usize][class as usize]
}

