use std::error::Error;
use sdl2::{
    video::Window,
    render::Canvas,
    rect::Rect,
    pixels::Color,
};

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr $(,)?) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

pub struct VideoDriver {
    canvas: Canvas<Window>,
}

impl VideoDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Result<Self, Box<dyn Error>> {
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("PSX-RS", 640, 480)
            .opengl()
            .build()?;

        let mut canvas = window.into_canvas()
            .index(find_sdl_gl_driver().ok_or("No opengl driver")?)
            .build()?;

        log::info!("SDL video subsystem initialized");

        canvas.set_draw_color(Color::RGB(145, 145, 135));
        canvas.fill_rect(rect!(0, 0, 640, 480))?;

        canvas.present();

        Ok( VideoDriver{ canvas })
    }
}

/* SDL Helpers */
fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            log::info!("opengl driver identified");
            return Some(index as u32);
        }
    }
    None
}