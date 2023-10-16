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
use tracing_subscriber::{filter, reload, prelude::*};

fn main() -> Result<()> {
    setup_trace();

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

fn setup_trace() {
    let filter = filter::LevelFilter::ERROR;
    let (filter, reload_handle) = reload::Layer::new(filter);
    let time_format = time::macros::format_description!(
        "[hour]:[minute]:[second].[subsecond digits:5]"
    );
    let time_offset = time::UtcOffset::current_local_offset()
        .unwrap_or_else(|_| time::UtcOffset::UTC);
    let timer = tracing_subscriber::fmt::time::OffsetTime::new(time_offset, time_format);
    let formatted_layer = tracing_subscriber::fmt::layer()
        .with_timer(timer);
    tracing_subscriber::registry()
        .with(filter)
        .with(formatted_layer)
        .init();

    unsafe { TRACING_RELOAD_HANDLE = Some(reload_handle); }
}
