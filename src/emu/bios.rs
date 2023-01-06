use std::{
    path::Path, 
    fs::File, 
    io::Read
};
use anyhow::{
    Result, 
    anyhow
};

pub const BIOS_SIZE: usize = 512 * 1024;
pub const BIOS_START:  u32 = 0x1fc0_0000;

pub struct Bios {
    data: Vec<u8>
}

impl Bios {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Bios { data }
    }

    pub fn get_rom(&self) -> &[u8] {
        &self.data
    }
}