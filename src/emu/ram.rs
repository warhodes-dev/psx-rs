//! Access executable RAM (read and write) 

use crate::emu::access::{Accessable, AccessWidth};

pub const RAM_SIZE : usize = 2 * 1024 * 1024;
pub const RAM_START: u32   = 0xa000_0000;
pub const RAM_END: u32     = RAM_START + RAM_SIZE as u32;

pub struct Ram {
    mem: Vec<u32>,
}

impl Ram {
    pub fn new() -> Self {
        let mem = vec![0xB0BA_CAFE; RAM_SIZE / 4];
        Ram { mem }
    }

    pub fn load<T: Accessable>(&self, offset: u32) -> T {
        log::trace!("ram.load(0x{offset:08x}) ({:?})", T::width());

        // Get value from correct byte subindex
        let word = self.mem[offset as usize >> 2];
        let sized_word = match T::width() {
            AccessWidth::Byte => {
                let shift = (offset & 3) * 8;
                (word >> shift) & 0xff
            }
            AccessWidth::Short => {
                let shift = (offset >> 1 & 1) * 16 ;
                (word >> shift) & 0xffff
            }
            AccessWidth::Long => word,
        };
        Accessable::from_u32(sized_word)
    }

    pub fn store<T: Accessable>(&mut self, offset: u32, val: T) {
        log::trace!("ram.store(0x{offset:08x}) ({:?})", T::width());

        // Shift value into correct byte subindex
        match T::width() {
            AccessWidth::Byte => {
                let word = self.mem[offset as usize / 4];
                let shift = (offset & 3) * 8;
                let diff = val.as_u32() << shift;
                let mask = 0xff << shift;

                let new_word = !mask & word | diff;
                self.mem[offset as usize / 4] = new_word;
            },
            AccessWidth::Short => {
                let word = self.mem[offset as usize / 4];
                let shift = (offset >> 1 & 1) * 16;
                let diff = val.as_u32() << shift;
                let mask = 0xffff << shift;

                let new_word = !mask & word | diff;
                self.mem[offset as usize / 4] = new_word;
            },
            AccessWidth::Long => {
                self.mem[offset as usize / 4] = val.as_u32();
            }
        }
    }
}