
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
    pub fn new(sdl_context: &Sdl) -> Result<Self, Box<dyn std::error::Error>> {
        let events = sdl_context.event_pump()?;
        log::info!("SDL input handler initialized");
        Ok(InputDriver{events})
    }
}