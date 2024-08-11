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

macro_rules! hi {
    ($expression:expr) => {
        // `stringify!` 把表达式*原样*转换成一个字符串。
        println!("{:?} = {:?}", stringify!($expression), $expression)
    };
}

#[macroquad::main("rboy")]
async fn main() {
    hi!(1 + 1);
    let cli = RboyCli::parse();

    if !Path::new(&cli.path).exists() {
        eprint!("file doesn't exist.")
    }
    graphic::set_gameboy_window_scale(cli.scale);
    // include_bytes!()
    let mut i = 0;
    loop {
        clear_background(RED);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        draw_circle(
            screen_width() - i as f32,
            screen_height() - 30.0,
            15.0,
            YELLOW,
        );
        i = (i + 1) % (screen_width() as i32);
        draw_text(
            format!("FPS: {:.2}", 1.0 / get_frame_time()).as_str(),
            0.,
            16.,
            32.,
            WHITE,
        );

        // draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);
        let now = Instant::now();

        next_frame().await;
        let elapsed_time = now.elapsed();
        // println!(
        //     "Running took {} ms.",
        //     elapsed_time.as_micros() as f32 / 1000.0
        // );
    }
}
