use std::path::Path;
use psx::{
    bios::Bios,
    cpu::Cpu,
    ic::Interconnect,
};

mod psx;

fn main() {
    match Bios::new(Path::new("./scph1001.bin")) {
        Err(e) => println!("Error: {}", e),
        Ok(bios) => {
            let ic = Interconnect::new(bios);
            let mut cpu = Cpu::new(ic);
            loop {
                cpu.run_next_instruction();
            }
        }
    }
}
