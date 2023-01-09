pub mod bios;
pub mod cpu;
pub mod xmem;
pub mod map;

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

    /// Routes load request @ addr to proper device
    pub fn load32(&self, addr: u32) -> u32 {
        if addr % 4 != 0 {
            panic!("unaligned load32 at address 0x{addr:08x}");
        }

        log::trace!("psx.load32(0x{addr:08x})");

        match map::get_region(addr) {
            map::MemRegion::Bios(base_addr)    => {
                let offset = addr - base_addr;
                return self.xmem.bios_load(offset);
            },
            map::MemRegion::MemCtrl(base_addr) => {
                log::warn!("read from memctrl region, but this is unimplemented");
                return 0;
            },
        }
    }

    pub fn store32(&self, addr: u32, val: u32) {
        if addr % 4 != 0 {
            panic!("unaligned store32 at address 0x{addr:08x}");
        }

        log::trace!("psx.store32(0x{addr:08x}, {val}");

        match map::get_region(addr) {
            map::MemRegion::Bios(base_addr)    => {
                panic!("attempt to write to bios region which is read only");
            },
            map::MemRegion::MemCtrl(base_addr) => {
                log::warn!("wrote to memctrl region, but this is unimplemented");
            },
        }
    }
}
