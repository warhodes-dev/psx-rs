pub mod bios;
pub mod cpu;
pub mod xmem;
pub mod map;
pub mod cop;

use crate::emu::{
    bios::Bios, 
    cpu::Cpu,
    xmem::XMemory,
    cop::Cop0,
};


pub struct Psx {
    bios: Bios,
    cpu: Cpu,
    cop0: Cop0,
    xmem: XMemory,
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
            cop0: Cop0::new(),
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
            map::Region::Bios(mapping) => {
                let offset = addr - mapping.base;
                return self.xmem.bios_load(offset);
            },
            map::Region::MemCtl(_mapping) => {
                log::warn!("read from memctrl region, but this is unimplemented");
                return 0;
            },
            map::Region::RamCtl(_mapping) => {
                log::warn!("read from ramctrl region, but this is unsupported");
                return 0;
            },
            map::Region::CacheCtl(_mapping) => {
                log::warn!("read from cachectrl region, but this is unsupported");
                return 0;
            }
        }
    }

    pub fn store32(&self, addr: u32, val: u32) {
        if addr % 4 != 0 {
            panic!("unaligned store32 at address 0x{addr:08x}");
        }

        log::trace!("psx.store32(0x{addr:08x}, {val})");

        match map::get_region(addr) {
            map::Region::Bios(_mapping) => {
                panic!("attempt to write to bios region which is read only");
            },
            map::Region::MemCtl(_mapping) => {
                log::warn!("wrote to memctrl region, but this is unimplemented");
            },
            map::Region::RamCtl(_mapping) => {
                log::warn!("wrote to memctrl region, but this is unimplemented");
            },
            map::Region::CacheCtl(_mapping) => {
                log::warn!("wrote to cachectrl region, but this is unsupported");
            }
        }
    }
}
