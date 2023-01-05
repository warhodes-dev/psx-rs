pub mod bios;
pub mod cpu;
pub mod xmem;

use crate::emu::{
    bios::Bios, 
    cpu::Cpu,
    xmem::XMemory,
};


pub struct Psx {
    bios: Bios,
    xmem: XMemory,
    cpu: Cpu,
}

impl Psx {
    pub fn new_from_bios(bios: Bios) -> Self {

        let bios_rom = bios.get_rom()
            .try_into()
            .expect(&format!("Bios size does not equal {:?}", bios::BIOS_SIZE));

        let mut xmem = XMemory::new();
        xmem.set_bios(bios_rom);

        Psx { 
            bios, 
            cpu: Cpu::new(),
            xmem,
        }
    }

    pub fn _new_from_disc(disc: (), bios: Bios) -> Self {

        let bios_rom = bios.get_rom()
            .try_into()
            .expect(&format!("Bios size does not equal {:?}", bios::BIOS_SIZE));

        let mut xmem = XMemory::new();
        xmem.set_bios(bios_rom);

        Psx {
            bios,
            cpu: Cpu::new(),
            xmem,
        }
    }

    pub fn read32(&self, addr: u32) -> u32 {
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
    pub const BIOS: MemRange = MemRange(BIOS_START, BIOS_START + BIOS_SIZE as u32);
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