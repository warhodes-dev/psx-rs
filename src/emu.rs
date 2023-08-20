//! Emulator core library

pub mod bios;
pub mod cpu;
pub mod map;
pub mod ram;
pub mod access;

use crate::emu::{
    bios::Bios, 
    cpu::Cpu,
    ram::Ram,
    access::{Accessable, AccessWidth},
};

/// Complete emulator core state. The contents of this struct comprise an accurate state
/// of a virtual PSX system.
pub struct Psx {
    bios: Bios,
    cpu: Cpu,
    ram: Ram,
    pub instruction_cnt: u64,
}

impl Psx {
    pub fn new_from_bios(bios_buf: &[u8; bios::BIOS_SIZE]) -> Self {
        Psx { 
            bios: Bios::new(bios_buf), 
            cpu: Cpu::new(),
            ram: Ram::new(),
            instruction_cnt: 0,
        }
    }

    /// Routes load request @ addr to proper device
    pub fn load<T: Accessable>(&self, addr: u32) -> T {
        tracing::trace!("psx.load(0x{addr:08x}) ({:?})", T::width());

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
            },
            map::Region::MemCtl(_mapping) => {
                tracing::warn!("read from memctrl region (0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            },
            map::Region::RamCtl(_mapping) => {
                tracing::warn!("read from ramctrl region (0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            },
            map::Region::IrqCtl(_mapping) => {
                tracing::warn!("read from irqctrl region (0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            },
            map::Region::Timer(_mapping) => {
                tracing::warn!("read from timer region (0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            },
            map::Region::CacheCtl(_mapping) => {
                tracing::warn!("read from cachectrl region (0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            },
            map::Region::Spu(_mapping) => {
                tracing::warn!("read from SPU memory region (0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            },
            map::Region::Exp1(_mapping) => {
                tracing::warn!("read from expansion region 1 (0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            },
            map::Region::Exp2(_mapping) => {
                tracing::warn!("read from expansion region 2 (0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            },
        }
    }

    pub fn store<T: Accessable>(&mut self, addr: u32, val: T) {
        tracing::trace!("psx.store(0x{addr:08x}, {}) ({:?})", val.as_u32(), T::width());

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
            },
            map::Region::MemCtl(_mapping) => {
                tracing::warn!("wrote to memctrl region (0x{addr:08x}), but this is unimplemented");
            },
            map::Region::RamCtl(_mapping) => {
                tracing::warn!("wrote to ramctrl region (0x{addr:08x}), but this is unimplemented");
            },
            map::Region::IrqCtl(_mapping) => {
                tracing::warn!("wrote to irqctrl region (0x{addr:08x}), but this is unimplemented");
            },
            map::Region::Timer(_mapping) => {
                tracing::warn!("wrote to timer region (0x{addr:08x}), but this is unimplemented");
            },
            map::Region::CacheCtl(_mapping) => {
                tracing::warn!("wrote to cachectrl region (0x{addr:08x}), but this is unsupported");
            },
            map::Region::Spu(_mapping) => {
                tracing::warn!("wrote to SPU memory region (0x{addr:08x}), but this is unsupported");
            },
            map::Region::Exp1(_mapping) => {
                tracing::warn!("wrote to expansion region 1 (0x{addr:08x}), but this is unsupported");
            },
            map::Region::Exp2(_mapping) => {
                tracing::warn!("wrote to expansion region 2 (0x{addr:08x}), but this is unsupported");
            }
        }
    }
}
