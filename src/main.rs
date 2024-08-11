use macroquad::prelude::*;
use miniquad::window::set_window_size;
use std::time::Instant;

fn window_conf() -> Conf {
    let scale = 2;
    Conf {
        window_title: "Window Conf".to_owned(),
        window_height: 144 * scale,
        window_width: 160 * scale,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    set_window_size(144, 144);
}
