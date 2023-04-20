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
    pretty_env_logger::init();
        //pretty_env_logger.formatted_builder()
        //.filter_level(log::LevelFilter::Trace)
        //.init();

    let mut ctx = Context::new(Path::new("./scph1001.bin"))?;
    ctx.run();
}
