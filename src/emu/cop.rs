use crate::emu::{
    Psx,
    cpu::RegisterIndex,
};

pub struct Cop0 {
    sr: u32,
}

impl Cop0 {
    pub fn new() -> Self {
        Cop0 {
            sr: 0,
        }
    }
    pub fn status(&self) -> ProcessorStatus {
        ProcessorStatus(self.sr)
    }
}

pub fn op_mtc0(psx: &mut Psx, cop_r: RegisterIndex, val: u32) {
    log::trace!("cop0 exec MTC0");
    match cop_r.into() {
        3 | 5 | 6 | 7 | 9 | 11 => {
            // TODO: What if non-zero is written?
            if val != 0 {
                panic!("unhandled write to cop0_r")
            }
        }
        12 => {
            psx.cop0.sr = val;
        },
        13 => {
            if val != 0 {
                panic!("unhandled write to CAUSE register");
            }
        }
        _else => panic!("unhandled cop0 register: 0x{_else:08x}"),
    } 
}

pub struct ProcessorStatus(u32);

impl ProcessorStatus {
    pub fn is_isolate_cache(self) -> bool {
        let ProcessorStatus(status) = self;
        status & 0x10000 != 0
    }
}
