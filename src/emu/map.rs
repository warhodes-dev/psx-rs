//!  Memory ranges for each memory map region

use std::ops::Range;
use lazy_static::lazy_static;
use iset::{interval_map, IntervalMap};

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

const RAM      : Mapping = Mapping::def(0x0000_0000, 2 * 1024 * 1024 /* 2 MB */);
const BIOS     : Mapping = Mapping::def(0x1fc0_0000, 512 * 1024 /* 512 KB */);
const MEM_CTL  : Mapping = Mapping::def(0x1f80_1000, 36);
const RAM_CTL  : Mapping = Mapping::def(0x1f80_1060, 4);
const CACHE_CTL: Mapping = Mapping::def(0xfffe_0130, 4);

lazy_static! {
    /// Contains the base address all memory intervals.
    static ref MEMORY_MAP: IntervalMap<u32, Region> = interval_map! {
        BIOS.range()      => Region::Bios(BIOS),
        RAM.range()       => Region::Ram(RAM),
        MEM_CTL.range()   => Region::MemCtl(MEM_CTL),
        RAM_CTL.range()   => Region::RamCtl(RAM_CTL),
        CACHE_CTL.range() => Region::CacheCtl(CACHE_CTL),
    };
}

/// Contains the base address of the associated region
#[derive(Copy, Clone)]
pub enum Region {
    Bios(Mapping),
    Ram(Mapping),
    MemCtl(Mapping),
    RamCtl(Mapping),
    CacheCtl(Mapping),
}

#[derive(Debug, Copy, Clone)]
pub struct Mapping {
    pub base: u32,
    pub size: usize,
}

impl Mapping {
    const fn def(base: u32, size: usize) -> Self {
        Mapping { base, size }
    }

    const fn range(&self) -> Range<u32> {
        self.base..(self.base + self.size as u32)
    }
}

pub fn get_region(addr: u32) -> Region {
    let query = addr..=addr;
    let query_result = MEMORY_MAP.iter(query).next()
        .expect(&format!("failed to look up addr 0x{addr:08x} in memory map: unknown region"))
        .1;
    *query_result
}

pub fn mask_region(addr: u32) -> u32 {
    let idx = (addr >> 29) as usize;
    addr & REGION_MASK[idx]
}