pub mod bios;
pub mod cpu;
pub mod map;
pub mod cop;
pub mod ram;
pub mod access;

use crate::emu::{
    bios::Bios, 
    cpu::Cpu,
    cop::Cop0,
    ram::Ram,
    access::{Accessable, AccessWidth},
};

pub struct Psx {
    bios: Bios,
    cpu: Cpu,
    cop0: Cop0,
    ram: Ram,
}

impl Psx {
    pub fn new_from_bios(bios_buf: &[u8; bios::BIOS_SIZE]) -> Self {
        Psx { 
            bios: Bios::new(bios_buf), 
            cpu: Cpu::new(),
            cop0: Cop0::new(),
            ram: Ram::new(),
        }
    }

    /// Routes load request @ addr to proper device
    pub fn load<T: Accessable>(&self, addr: u32) -> T {
        log::trace!("psx.load(0x{addr:08x}) ({:?})", T::width());

        if cfg!(debug_assertions) {
            if T::width() == AccessWidth::Word && addr % 4 != 0 {
                panic!("unaligned load<32> at address 0x{addr:08x}");
            }
            if T::width() == AccessWidth::Half && addr % 2 != 0 {
                panic!("unaligned load<16> at address 0x{addr:08x}");
            }
        }

        let paddr = map::mask_region(addr);
        match map::get_region(paddr) {
            map::Region::Bios(mapping) => {
                let offset = paddr - mapping.base;
                return self.bios.load::<T>(offset);
            },
            map::Region::Ram(mapping) => {
                let offset = paddr - mapping.base;
                return self.ram.load::<T>(offset);
            }
            map::Region::MemCtl(_mapping) => {
                log::warn!("read from memctrl region, but this is unsupported");
                return T::from_u32(0);
            },
            map::Region::RamCtl(_mapping) => {
                log::warn!("read from ramctrl region, but this is unsupported");
                return T::from_u32(0);
            },
            map::Region::CacheCtl(_mapping) => {
                log::warn!("read from cachectrl region, but this is unsupported");
                return T::from_u32(0);
            }
        }
    }

    pub fn store<T: Accessable>(&mut self, addr: u32, val: T) {
        log::trace!("psx.store(0x{addr:08x}, {}) ({:?})", val.as_u32(), T::width());

        if cfg!(debug_assertions) {
            if T::width() == AccessWidth::Word && addr % 4 != 0 {
                panic!("unaligned store<32> at address 0x{addr:08x}");
            }
            if T::width() == AccessWidth::Half && addr % 2 != 0 {
                panic!("unaligned store<16> at address 0x{addr:08x}");
            }
        }

        let paddr = map::mask_region(addr);
        match map::get_region(paddr) {
            map::Region::Bios(_mapping) => {
                panic!("attempt to write to bios region which is read only");
            },
            map::Region::Ram(mapping) => {
                let offset = paddr - mapping.base;
                self.ram.store::<T>(offset, val);
            }
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
