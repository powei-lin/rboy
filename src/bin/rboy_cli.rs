use clap::Parser;
use macroquad::prelude::*;
use rboy::{core, graphic};
use std::path::Path;
use std::time::Instant;

use macroquad::ui::{
    hash, root_ui,
    widgets::{self, Group},
    Drag, Ui,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct RboyCli {
    /// path to .gb
    path: String,

    #[arg(short, long, default_value_t = 4)]
    scale: u8,

    #[arg(short, long, action)]
    debug: bool,
}

#[macroquad::main("rboy")]
async fn main() {
    let a = 128u8;
    println!("xor a {}", a << 1);

    let cli = RboyCli::parse();

    if !Path::new(&cli.path).exists() {
        eprint!("file doesn't exist.")
    }

    let mut gameboy_core = rboy::core::Core::new(true);
    gameboy_core.load_game_rom(&cli.path);
    let mut screen = graphic::Screen::new(cli.scale, cli.debug);

    let mut i = 0;
    let mut count = 0;
    loop {
        gameboy_core.tick();
        println!(
            "----------------------------------\n{} {}",
            count, gameboy_core.cpu
        );
        count += 1;
        clear_background(LIGHTGRAY);

        let r = i / core::constants::LCD_WIDTH;
        let c = i % core::constants::LCD_WIDTH;
        screen.update_pixel_in_buffer(c, r, 0);
        screen.draw_frame();
        i = (i + 1) % graphic::GAMEBOY_WINDOW_PIXELS;
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
