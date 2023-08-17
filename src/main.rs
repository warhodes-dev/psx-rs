
use std::path::Path;
use anyhow::Result;

#[allow(unused_imports)]
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
