use crate::bios::Bios;



pub struct Interconnect {
    bios: Bios,
}

impl Interconnect {
    pub fn new(bios: Bios) -> Self {
        Interconnect { 
            bios, 
        }
    }

    pub fn load32(&self, addr: u32) -> u32 {
        if let Some(offset) = mem_map::BIOS_RANGE.contains(addr) {
            return self.bios.load32(offset);
        } else {
            panic!("unhandled fetch 32 at address {:08x}", addr);
        }

    }
}

mod mem_map {
    use crate::bios::{BIOS_START, BIOS_SIZE};

    pub const BIOS_RANGE: MemRange = MemRange(BIOS_START, BIOS_START + BIOS_SIZE);

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