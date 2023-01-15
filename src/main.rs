#![allow(unused_imports)]

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
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let ctx = Context::new_no_disc(Path::new("./scph1001.bin"))?;
    let mut psx = *ctx.psx;

    for i in 0..300 {
        log::debug!("=== Instruction {i:2} issued ===");
        cpu::handle_next_instruction(&mut psx);
    }
    
    Ok(())
}
