mod bios;
mod cpu;
mod ic;

use std::path::Path;
use bios::Bios;
use cpu::Cpu;
use ic::Interconnect;

fn main() {
    match Bios::new(Path::new("./scph1001.bin")) {
        Err(e) => println!("Error: {}", e),
        Ok(bios) => {
            let ic = Interconnect::new(bios);
            let mut cpu = Cpu::new(ic);
            loop {
                cpu.step();
            }
        }
    }
}
