//! Module for handling database of bios versions

pub const BIOS_SIZE : usize = 512 * 1024;
pub const BIOS_START: u32   = 0x1fc0_0000;
pub const BIOS_END  : u32   = BIOS_START + BIOS_SIZE as u32;

pub struct Bios {
    mem: Vec<u32>
}

impl Bios {
    pub fn new(buf: &[u8; BIOS_SIZE])-> Self {

        let mem = buf.chunks(4)
            .map(|chunk| {
                if let Ok(array_chunk) = chunk.try_into() {
                    u32::from_ne_bytes(array_chunk)
                } else {
                    panic!("Failed to create u32 from u8. This should never happen")
                }
            })
            .collect::<Vec<u32>>();

        /*
        let mem = buf.array_chunks::<4>()
            .map(|chunk| u32::from_ne_bytes(*chunk))
            .collect::<Vec<u32>>();
        */

        Bios { mem }
    }

    pub fn get_rom(&self) -> &[u32] {
        &self.mem
    }

    fn load(&self, offset: usize) -> u32 {
        self.mem[offset / 4]
    }

    pub fn load32(&self, offset: u32) -> u32 {
        log::trace!("bios.load32(0x{offset:08x})");
        self.load(offset as usize)
    }
}