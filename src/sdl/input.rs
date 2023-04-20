use anyhow::{anyhow, Result};
use sdl2::{
    event::Event,
    keyboard::Keycode,
    EventPump,
    Sdl,
};

pub struct InputDriver {
    events: EventPump,
}

impl InputDriver {
    pub fn new(sdl_context: &Sdl) -> Result<Self> {
        let events = sdl_context.event_pump().map_err(|e| anyhow!(e))?;
        log::info!("SDL input handler initialized");
        Ok(InputDriver{events})
    }
}