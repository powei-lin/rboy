use clap::Parser;
use macroquad::prelude::*;
use rboy::graphic;
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct RboyCli {
    /// path to .gb
    path: String,

    #[arg(short, long, default_value_t = 4)]
    scale: u8,
}

#[macroquad::main("rboy")]
async fn main() {
    let cli = RboyCli::parse();

    if !Path::new(&cli.path).exists() {
        eprint!("file doesn't exist.")
    }

    let mut screen = graphic::Screen::new(cli.scale, false);

    let mut i = 0;
    loop {
        clear_background(LIGHTGRAY);

        let r = i / graphic::GAMEBOY_WINDOW_WIDTH;
        let c = i % graphic::GAMEBOY_WINDOW_WIDTH;
        screen.update_pixel_in_buffer(c, r, 0);
        screen.draw_frame();
        i = (i + 1) % (graphic::GAMEBOY_WINDOW_HEIGHT * graphic::GAMEBOY_WINDOW_WIDTH);
        draw_text(
            format!("FPS: {:.2}", 1.0 / get_frame_time()).as_str(),
            0.,
            16.,
            32.,
            WHITE,
        );

        next_frame().await;
    }
}
