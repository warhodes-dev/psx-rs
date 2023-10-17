use std::path::Path;

use tracing_subscriber::{filter, reload, prelude::*};
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
    config::{self, Config},
};

fn main() -> Result<()> {
    let config = config::Config::parse_args();
    setup_trace(&config);

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

fn setup_trace(config: &Config) {
    let filter = match config.log_level {
        config::LogLevel::Trace => filter::LevelFilter::TRACE,
        config::LogLevel::Debug => filter::LevelFilter::DEBUG,
        config::LogLevel::Info  => filter::LevelFilter::INFO,
        config::LogLevel::Warn  => filter::LevelFilter::WARN,
        config::LogLevel::Error => filter::LevelFilter::ERROR,
    };
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
