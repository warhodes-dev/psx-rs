//! Coprocessor module for handling internal coprocessor routines

use crate::emu::{
    Psx,
    cpu::instruction::RegisterIndex,
    cpu::exception::ExceptionVector,
};

#[derive(Default, Debug)]
pub struct Cop0 {
    pub sr: u32,
    pub cause: u32,
    pub epc: u32,
}

impl Cop0 {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn status(&self) -> ProcessorStatus {
        ProcessorStatus(self.sr)
    }
}

pub fn mtc0(psx: &mut Psx, cop_r: RegisterIndex, val: u32) {
    log::trace!("cop0 exec MTC0");
    match cop_r.into() {
        3 | 5 | 6 | 7 | 9 | 11 => {
            if val != 0 {
                panic!("unhandled write to cop0_r")
            }
        }
        12 => {
            psx.cpu.cop.sr = val;
        },
        13 => {
            if val != 0 {
                panic!("unhandled write to CAUSE register");
            }
        },
        14 => {
            if val != 0 {
                panic!("unhandled write to EPC register")
            }
        }
        _else => panic!("unhandled write to cop0 register {_else}"),
    } 
}

pub fn mfc0(psx: &mut Psx, cop_r: RegisterIndex) -> u32 {
    log::trace!("cop0 exec MFC0");
    match cop_r.into() {
        12 => psx.cpu.cop.sr,
        13 => psx.cpu.cop.cause,
        14 => psx.cpu.cop.epc,
        _else => panic!("Unhandled read from cop0 register {_else}")
    }
}

pub struct ProcessorStatus(u32);

impl ProcessorStatus {
    pub fn is_isolate_cache(&self) -> bool {
        let ProcessorStatus(status) = self;
        status & 0x10000 != 0
    }
    pub fn boot_exception_vector(&self) -> ExceptionVector {
        match self.0 & (1 << 22) == 1 {
            true => ExceptionVector::Boot,
            false => ExceptionVector::Normal,
        }
    }
}

