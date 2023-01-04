use std::path::Path;
use anyhow::Result;
use psx_rs::{
    Context,
    emu::{
        bios::Bios,
        cpu::{Cpu, self},
        Psx,
    },
};

fn main() -> Result<()> {

    simple_logger::SimpleLogger::new()
        .init()?;

    let ctx = Context::new_no_disc(Path::new("./scph1001.bin"))?;
    let mut psx = *ctx.psx;

    for _ in 0..2 {
        cpu::handle_instruction(&mut psx);
    }
    

    /*
    match Bios::new(Path::new("./scph1001.bin")) {
        Err(e) => println!("Error: {}", e),
        Ok(bios) => {
            let mut psx = Psx::new(bios);
            cpu::handle_instruction(&mut psx);
        }
    }
    */
    Ok(())
}
