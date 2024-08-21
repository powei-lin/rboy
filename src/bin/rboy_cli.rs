use clap::Parser;
use macroquad::prelude::*;
use rboy::core::constants::{LCD_HEIGHT, LCD_WIDTH};
use rboy::{core, graphic};
use std::path::Path;
use std::{thread, time};

const WINDOW_SCALE: u8 = 2;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct RboyCli {
    /// path to .gb
    path: String,

    #[arg(short, long, default_value_t = WINDOW_SCALE)]
    scale: u8,

    #[arg(short, long, action)]
    debug: bool,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Window Conf".to_owned(),
        window_height: LCD_HEIGHT as i32 * WINDOW_SCALE as i32,
        window_width: LCD_WIDTH as i32 * WINDOW_SCALE as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let cli = RboyCli::parse();

    if !Path::new(&cli.path).exists() {
        eprint!("file doesn't exist.")
    }

    let mut gameboy_core = rboy::core::Core::new(true);
    gameboy_core.load_game_rom(&cli.path);
    let screen = graphic::Screen::new(cli.scale, cli.debug);

    let mut i = 0;
    let mut count = 0;

    // let one_sixtyth = time::Duration::from_secs_f64(1.0 / 60.0);
    // let mut now = time::Instant::now();

    loop {
        // println!(
        //     "----------------------------------\n{} {}",
        //     count, gameboy_core.cpu
        // );
        count += 1;
        if gameboy_core.tick() {
            clear_background(LIGHTGRAY);

            screen.draw_frame();
            screen.draw_bg_frame(gameboy_core.get_bg_frame_buffer());
            screen.draw_tiles_frame(gameboy_core.get_tiles_frame_buffer());
            if cli.debug {
                draw_text(
                    format!("FPS: {:.2}", 1.0 / get_frame_time()).as_str(),
                    0.,
                    16.,
                    32.,
                    BLACK,
                );
            }
            next_frame().await;
        }
        // println!("count {}", count);

        // let r = i / core::constants::LCD_WIDTH;
        // let c = i % core::constants::LCD_WIDTH;
        // screen.update_pixel_in_buffer(c, r, 0);
        // screen.draw_frame();
        // i = (i + 1) % graphic::GAMEBOY_WINDOW_PIXELS;
    }
}
