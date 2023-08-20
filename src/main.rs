use std::path::Path;

use lazy_static::lazy_static;
use anyhow::Result;

#[allow(unused_imports)]
use psx_rs::{
    TRACING_RELOAD_HANDLE,
    Context,
    emu::{
        bios::Bios,
        cpu::{Cpu, self},
        Psx,
    },
};
use tracing_subscriber::{filter, reload, prelude::*, fmt};

fn main() -> Result<()> {
    //pretty_env_logger::init();
        //pretty_env_logger.formatted_builder()
        //.filter_level(log::LevelFilter::Trace)
        //.init();

    let filter = filter::LevelFilter::ERROR;
    let (filter, reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    unsafe { TRACING_RELOAD_HANDLE = Some(reload_handle); }

    let start = std::time::SystemTime::now();
    let mut ctx = Context::new(Path::new("./scph1001.bin"))?;
    let any_error = ctx.run();
    let done = start.elapsed()?.as_millis();
    println!("Elapsed time: {done}");
    if let Err(err) = any_error {
        println!("Error halted operation: {err}");
    }
    println!("Total instructions processed: {}", ctx.psx.instruction_cnt);
    Ok(())
}
