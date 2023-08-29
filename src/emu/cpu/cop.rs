//! Coprocessor module for handling internal coprocessor routines

use crate::emu::{
    Psx,
    cpu::{
        instruction::RegisterIndex,
        exception::{Exception, ExceptionVector},
    },
    Bus,
};

#[derive(Default, Debug)]
pub struct Cop0 {
    /// Status register
    sr: u32,
    /// Cause (exception) register
    cause: u32,
    /// Exception program counter
    pub epc: u32,
}

impl Cop0 {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn status(&self) -> ProcessorStatus {
        ProcessorStatus(self.sr)
    }

    /// Disables interrupts and sets cpu to kernel mode
    //
    // Stores previous processor mode in mode stack.
    //
    // From emulation guide, pg. 66:
    //
    // Shift bits[5:0] of ‘SR‘ two places to the left.
    // Those bits are three pairs of InterruptEnable/
    // UserMode bits behaving like a stack 3 entries 
    // deep.
    pub fn push_mode(&mut self) {
        
        let mode = self.sr & 0x3f;
        self.sr &= !0x3f;
        self.sr |= (mode << 2) & 0x3f;
    }

    pub fn pop_mode(&mut self) {
        let mode = self.sr & 0x3f;
        self.sr &= !0x3f;
        self.sr |= mode >> 2;
    }

    pub fn set_cause(&mut self, cause: Exception) {
        self.cause &= !0x7c;
        self.cause |= (cause as u32) << 2;
    }

    /// COP0 internal implementation of MTC0: Move to coprocessor 0
    pub fn mtc0(&mut self, bus: &mut Bus, cop_r: RegisterIndex, val: u32) {
        tracing::trace!("cop0 exec MTC0");
        match cop_r.into() {
            3 | 5 | 6 | 7 | 9 | 11 => {
                if val != 0 {
                    panic!("unhandled write to cop0_r")
                }
            }
            12 => {
                self.sr = val;
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

    /// COP0 internal implementation of MFC0: Move from coprocessor 0
    pub fn mfc0(&mut self, bus: &mut Bus, cop_r: RegisterIndex) -> u32 {
        tracing::trace!("cop0 exec MFC0");
        match cop_r.into() {
            12 => self.sr,
            13 => self.cause,
            14 => self.epc,
            _else => panic!("Unhandled read from cop0 register {_else}")
        }
    }
}



pub struct ProcessorStatus(u32);

impl ProcessorStatus {
    pub fn is_isolate_cache(&self) -> bool {
        let ProcessorStatus(status) = self;
        status & 0x10000 != 0
    }
    pub fn exception_vector(&self) -> ExceptionVector {
        match self.0 & (1 << 22) != 0 {
            true => ExceptionVector::Boot,
            false => ExceptionVector::Normal,
        }
    }
}

