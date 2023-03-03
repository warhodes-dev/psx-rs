

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

    // TODO: Generalize this to support u8, u16, and u32
    fn load(&self, offset: usize) -> u32 {
        self.mem[offset / 4]
    }

    pub fn load32(&self, offset: u32) -> u32 {
        log::trace!("ram.load32(0x{offset:08x})");
        self.load(offset as usize)
    }

    fn store(&mut self, offset: usize, val: u32) {
        self.mem[offset / 4] = val;
    }

    pub fn store32(&mut self, offset: u32, val: u32) {
        self.store(offset as usize, val);
    }
}

/*
impl Ram {
    pub fn new() -> Self {
        let mem = vec![0xffff_ffff; RAM_SIZE]; 
        Ram {
            mem,
        }
    }

    pub fn set_ram(&mut self, bios: &[u8; BIOS_SIZE]) {
        let mut bios_u32: Vec<u32> = Vec::new();

        for chunk in bios.chunks(4) {
            let quad = chunk.try_into().expect("Bios size must be a multiple of 4");
            let word = u32::from_le_bytes(quad);
            bios_u32.push(word);
        }

        let (bios_region, _ram_region) = self.regions();
        bios_region.copy_from_slice(bios_u32.as_slice());
    }

    fn store(&mut self, offset: usize, val: u32) {
        self.mem[offset / 4] = val;
    }

    fn load(&self, offset: usize) -> u32 {
        self.mem[offset / 4]
    }

    pub fn ram_load(&self, offset: u32) ->  u32 {
        let offset = RAM_OFFSET + offset as usize;
        self.load(offset)
    }

    pub fn ram_store(&mut self, offset: u32, val: u32) {
        let offset = RAM_OFFSET + offset as usize;
        self.store(offset, val);
    }

    pub fn bios_load(&self, offset: u32) -> u32 {
        log::trace!("bios_load(0x{offset:08x})");
        let offset = BIOS_OFFSET + offset as usize;
        self.load(offset)
    }

    fn regions(&mut self) -> (&mut [u32], &mut [u32]) {
        self.mem.split_at_mut(BIOS_SIZE / 4)
    }
}
*/