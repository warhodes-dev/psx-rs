pub mod bios;
pub mod cpu;
pub mod xmem;

use crate::emu::{
    bios::Bios, 
    cpu::Cpu
};

pub struct Psx {
    bios: Bios,
    cpu: Cpu,
}

impl Psx {
    pub fn new_from_bios(bios: Bios) -> Self {
        Psx { 
            bios, 
            cpu: Cpu::new(),
        }
    }

    pub fn _new_from_disc(disc: (), bios: Bios) -> Self {
        Psx {
            bios,
            cpu: Cpu::new(),
        }
    }

    pub fn load32(&self, addr: u32) -> u32 {
        if let Some(offset) = mmap::BIOS.contains(addr) {
            return self.bios.load32(offset);
        } else {
            panic!("unhandled fetch 32 at address {:08x}", addr);
        }

    }
}

mod mmap {
    use crate::emu::bios::{BIOS_START, BIOS_SIZE};

    //  Memory ranges for each memory map region
    /// Memory map region for BIOS ROM
    pub const BIOS: MemRange = MemRange(BIOS_START, BIOS_START + BIOS_SIZE);
    /// Memory map region for x
 // pub const X: MemRange = MemRange(X_START, X_START + X_SIZE)

    // IDEA: try interval-tools
    pub struct MemRange(u32, u32);

    impl MemRange {
        pub fn contains(self, addr: u32) -> Option<u32> {
            let MemRange(start, end) = self;

            if addr >= start && addr < end {
                Some(addr - start)
            } else {
                None
            }
        }
    }
}