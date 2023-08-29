//! Module for handling bios access and database of bios versions

use crate::emu::access::{AccessWidth, Access};

pub const BIOS_SIZE : usize = 512 * 1024;
pub const BIOS_START: u32   = 0xbfc0_0000;
pub const BIOS_END  : u32   = BIOS_START + BIOS_SIZE as u32;

pub struct Bios {
    mem: Vec<u32>
}

impl Bios {
    pub fn new(buf: &[u8; BIOS_SIZE])-> Self {
        let mem = buf.array_chunks::<4>()
            .map(|chunk| u32::from_ne_bytes(*chunk))
            .collect::<Vec<u32>>();

        Bios { mem }
    }

    pub fn get_rom(&self) -> &[u32] {
        &self.mem
    }

    pub fn load<T: Access>(&self, offset: u32) -> T {
        tracing::trace!("bios.load(0x{offset:08x}) ({:?})", T::width());

        // Get value from correct byte subindex
        let word = self.mem[offset as usize / 4];
        let sized_word = match T::width() {
            AccessWidth::Byte => {
                let shift = (offset & 3) * 8;
                (word >> shift) & 0xff
            }
            AccessWidth::Half => {
                let shift = (offset >> 1 & 1) * 16 ;
                (word >> shift) & 0xffff
            }
            AccessWidth::Word => word,
        };
        Access::from_u32(sized_word)
    }
}