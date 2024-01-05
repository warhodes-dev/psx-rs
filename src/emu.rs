//! Emulator core library

pub mod bios;
pub mod cpu;
pub mod map;
pub mod ram;
pub mod access;
pub mod bus;

use crate::emu::{
    bios::Bios, 
    cpu::Cpu,
    ram::Ram,
    bus::Bus,
    access::{Access, AccessWidth},
};

/// Complete emulator core state. The contents of this struct comprise an accurate state
/// of a virtual PSX system.
pub struct Psx {
    cpu: Cpu,
    bus: Bus,
    pub instructions_retired: u64,
}

impl Psx {
    pub fn new_from_bios(bios_buf: &[u8; bios::BIOS_SIZE]) -> Self {
        let bios = Bios::new(bios_buf);
        let ram = Ram::new();

        let bus = Bus::new(ram, bios);

        Psx { 
            bus,
            cpu: Cpu::new(),
            instructions_retired: 0,
        }
    }
    pub fn step(&mut self) {
        self.instructions_retired += 1;
        self.cpu.handle_next_instruction(&mut self.bus);
    }
}
