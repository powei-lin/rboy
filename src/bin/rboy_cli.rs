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

fn window_conf() -> Conf {
    Conf {
        window_title: "rboy".to_owned(),
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

    graphic::set_gameboy_window_scale(cli.scale);

    let (window_width, window_height) = graphic::gameboy_window_size(cli.scale);
    let rgba_pixels = (window_height * window_width * 4) as usize;
    let mut frame_buffer: Vec<u8> = Vec::from_iter(std::iter::repeat(255).take(rgba_pixels));
    let texture = Texture2D::from_rgba8(
        window_width as u16,
        window_height as u16,
        frame_buffer.as_slice(),
    );

    let mut i = 0;
    loop {
        clear_background(LIGHTGRAY);

        let r = i / graphic::GAMEBOY_WINDOW_WIDTH;
        let c = i % graphic::GAMEBOY_WINDOW_WIDTH;
        for rr in (r * cli.scale as u32)..((r + 1) * cli.scale as u32) {
            for cc in (c * cli.scale as u32)..((c + 1) * cli.scale as u32) {
                let idx = ((rr * window_width + cc) * 4) as usize;
                frame_buffer[idx] = 0;
                frame_buffer[idx + 1] = 0;
                frame_buffer[idx + 2] = 0;
            }
        }
        // // i = (i + 4) % (graphic::GAMEBOY_WINDOW_PIXELS as usize * 4);
        texture.update_from_bytes(window_width, window_height, &frame_buffer);

        draw_texture(&texture, 0.0, 0.0, WHITE);
        i = (i + 1) % (graphic::GAMEBOY_WINDOW_HEIGHT * graphic::GAMEBOY_WINDOW_WIDTH);
        // draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        // draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        // draw_circle(
        //     screen_width() - i as f32,
        //     screen_height() - 30.0,
        //     15.0,
        //     YELLOW,
        // );
        // i = (i + 1) % (screen_width() as i32);
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
