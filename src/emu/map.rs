use lazy_static::lazy_static;
use iset::{interval_map, IntervalMap};
use crate::emu::bios::{BIOS_START, BIOS_SIZE, BIOS_END};

//  Memory ranges for each memory map region
/// Memory map region for BIOS ROM
pub const BIOS: MemRange = MemRange(BIOS_START, BIOS_START + BIOS_SIZE as u32);

/// Memory map region for x
// pub const X: MemRange = MemRange(X_START, X_START + X_SIZE)
pub const MEMCTRL: MemRange = MemRange(MEMCTRL_START, MEMCTRL_START + MEMCTRL_SIZE as u32);
const MEMCTRL_SIZE : usize = 36;
const MEMCTRL_START: u32   = 0x1f801000;
const MEMCTRL_END  : u32   = MEMCTRL_START + MEMCTRL_SIZE as u32;

/// Memory map region for x
// pub const X: MemRange = MemRange(X_START, X_START + X_SIZE)

pub struct MemRange(u32, u32);

impl MemRange {
    pub fn _contains(self, addr: u32) -> Option<u32> {

        let MemRange(start, end) = self;
        log::debug!("checking if 0x{addr:08x} is contained in 0x{start:08x}-0x{end:08x}");

        if addr >= start && addr < end {
            Some(addr - start)
        } else {
            None
        }
    }
}

pub fn get_region(addr: u32) -> MemRegion {
    let query = addr..=addr;
    let query_result = BASE_MAP.iter(query).next()
        .expect("failed to look up addr 0x{addr:08x} in memory map: unknown region")
        .1;
    *query_result
}

/// Contains the base address of the associated region
#[derive(Copy, Clone)]
pub enum MemRegion {
    Bios(u32),
    MemCtrl(u32),
}

lazy_static! {
    /// Contains the base address all memory intervals.
    static ref BASE_MAP: IntervalMap<u32, MemRegion> = interval_map! {
        BIOS_START..BIOS_END       => MemRegion::Bios(BIOS_START),
        MEMCTRL_START..MEMCTRL_END => MemRegion::MemCtrl(MEMCTRL_START),
    };
}