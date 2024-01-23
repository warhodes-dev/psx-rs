

use crate::{emu::{
    map,
    Ram,
    Bios,
    Access, AccessWidth
}, set_log_level};

pub struct Bus {
    ram: Ram,
    bios: Bios,
}
impl Bus {
    pub fn new(
        ram: Ram,
        bios: Bios,
    ) -> Self {
        Bus {
            ram,
            bios,
        }
    }

    /// Routes load request @ addr to proper device
    pub fn load<T: Access>(&self, addr: u32) -> T {
        tracing::trace!("psx.load(0x{addr:08x}) ({:?})", T::width());

        /* Handled via exceptions in cpu.exception()
        if cfg!(debug_assertions) {
            if T::width() == AccessWidth::Word && addr % 4 != 0 {
                panic!("unaligned load<32> at address 0x{addr:08x}");
            }
            if T::width() == AccessWidth::Half && addr % 2 != 0 {
                panic!("unaligned load<16> at address 0x{addr:08x}");
            }
        }
        */

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
                tracing::warn!("read from SPU region (0x{addr:08x}), but this is unimplemented");
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
            map::Region::Dma(_mapping) => {
                tracing::warn!("read from dma register 0x{addr:08x}), but this is unimplemented");
                return T::from_u32(0);
            }
            map::Region::Gpu(mapping) => {
                tracing::warn!("read from gpu register 0x{addr:08x}), but this is unimplemented");
                let offset = paddr - mapping.base;
                // Simulate GPUSTAT::Ready to recieve DMA blocks
                let response = match offset {
                    4 => 0x1000_0000,
                    _ => 0,
                };
                T::from_u32(response)
            }
        }
    }

    pub fn store<T: Access>(&mut self, addr: u32, val: T) {
        tracing::trace!("psx.store(0x{addr:08x}, {}) ({:?})", val.as_u32(), T::width());

        /* Handled via exceptions in cpu.exception()
        if cfg!(debug_assertions) {
            if T::width() == AccessWidth::Word && addr % 4 != 0 {
                panic!("unaligned store<32> at address 0x{addr:08x}");
            }
            if T::width() == AccessWidth::Half && addr % 2 != 0 {
                panic!("unaligned store<16> at address 0x{addr:08x}");
            }
        }
        */

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
                tracing::warn!("wrote to SPU region (0x{addr:08x}), but this is unsupported");
            },
            map::Region::Exp1(_mapping) => {
                tracing::warn!("wrote to expansion region 1 (0x{addr:08x}), but this is unsupported");
            },
            map::Region::Exp2(_mapping) => {
                tracing::warn!("wrote to expansion region 2 (0x{addr:08x}), but this is unsupported");
            },
            map::Region::Dma(_mapping) => {
                tracing::warn!("wrote to dma register (0x{addr:08x}), but this is unsupported");
            },
            map::Region::Gpu(_mapping) => {
                tracing::warn!("wrote to gpu register (0x{addr:08x}), but this is unsupported");
            },
        }
    }
}