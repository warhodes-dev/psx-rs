//!  Memory ranges for each memory map region

use std::ops::Range;
use lazy_static::lazy_static;

const REGION_MASK: [u32; 8] = [
    // KUSEG: 2048 MB
    0xffff_ffff, 0xffff_ffff, 0xffff_ffff, 0xffff_ffff,
    // KSEG0:  512 MB
    0x7fff_ffff,
    // KSEG1:  512 MB
    0x1fff_ffff,
    // KSEG2: 1024 MB
    0xffff_ffff, 0xffff_ffff,
];

//TODO: Divide this up into real regions and emulated regions.
//
//      Main RAM is a real region:  
//
//            KUSEG        KSEG0        KSEG1
//            0x0000_0000, 0x8000_0000, 0xA000_0000
//
//      CACHE_CTL (here as just a 4 byte region) is actually
//      part of 'Internal CPU control registers' region: 
//
//            KSEG2
//            0xfffe_0000
//                                      Base addr    Size in bytes
//                                      -----------  -------------------------
const RAM      : Mapping = Mapping::new(0x0000_0000, n_mib_bytes!(2) as u32);
const BIOS     : Mapping = Mapping::new(0x1fc0_0000, n_kib_bytes!(512) as u32);
const MEM_CTL  : Mapping = Mapping::new(0x1f80_1000, 36);
const RAM_CTL  : Mapping = Mapping::new(0x1f80_1060, 4);
const IRQ_CTL  : Mapping = Mapping::new(0x1f80_1070, 8);
const TIMER    : Mapping = Mapping::new(0x1f80_1100, 48);
const CACHE_CTL: Mapping = Mapping::new(0xfffe_0130, 4);
const SPU      : Mapping = Mapping::new(0x1f80_1c00, 640);
const EXP1     : Mapping = Mapping::new(0x1f00_0000, n_kib_bytes!(8) as u32);
const EXP2     : Mapping = Mapping::new(0x1f80_2000, n_kib_bytes!(8) as u32);
const DMA      : Mapping = Mapping::new(0x1f80_1080, 128);

/// Contains the base address of the associated region
#[derive(Copy, Clone)]
pub enum Region {
    Bios(Mapping),
    Ram(Mapping),
    MemCtl(Mapping),
    RamCtl(Mapping),
    IrqCtl(Mapping),
    Timer(Mapping),
    CacheCtl(Mapping),
    Spu(Mapping),
    Exp1(Mapping),
    Exp2(Mapping),
    Dma(Mapping),
}

// TODO: organize these based on profiling data?
// TODO: TODO: organize these dynamically based on profiling data?
const MEMORY_MAP: [(Mapping, Region); 11] = [
    (RAM,       Region::Ram(RAM)),
    (BIOS,      Region::Bios(BIOS)),
    (MEM_CTL,   Region::MemCtl(MEM_CTL)),
    (RAM_CTL,   Region::RamCtl(RAM_CTL)),
    (IRQ_CTL,   Region::IrqCtl(IRQ_CTL)),
    (TIMER,     Region::Timer(TIMER)),
    (CACHE_CTL, Region::CacheCtl(CACHE_CTL)),
    (SPU,       Region::Spu(SPU)),
    (EXP1,      Region::Exp1(EXP1)),
    (EXP2,      Region::Exp1(EXP2)),
    (DMA,       Region::Dma(DMA)),

];

#[derive(Debug, Copy, Clone)]
pub struct Mapping {
    pub base: u32,
    pub size: u32,
}

impl Mapping {
    const fn new(base: u32, size: u32) -> Self {
        Mapping { 
            base, 
            size,
        }
    }

    const fn range(&self) -> Range<u32> {
        self.base..(self.base + self.size)
    }

    /// Return `Some(offset)` if addr is contained in mapping
    fn contains(self, addr: u32) -> bool {
        addr >= self.base && addr < self.base + self.size
    }
}

pub fn get_region(addr: u32) -> Region {
    for (mapping, region) in MEMORY_MAP {
        if mapping.contains(addr) {
            return region;
        }
    }
    panic!("Unknown region @ {addr:08x}");
}

pub fn mask_region(addr: u32) -> u32 {
    let idx = (addr >> 29) as usize;
    addr & REGION_MASK[idx]
}