mod audio;
mod collision;
mod config;
mod effects;
mod entities;
mod game;
mod input;
mod score;
mod spawner;
mod synth;
mod theme;
mod ui;
mod world;

use macroquad::prelude::*;

use audio::Sfx;
use config::{SCREEN_HEIGHT, SCREEN_WIDTH};
use game::Game;

// ==========================================
// CẤU HÌNH CỬA SỔ & VÒNG LẶP CHÍNH
// ==========================================
fn window_conf() -> Conf {
    Conf {
        window_title: "Rust Dino Runner - Macroquad".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Khởi tạo hạt giống ngẫu nhiên dựa trên thời gian thực
    // (now() trả về giây dạng f64, nhân 1000 để seed đổi theo mili-giây)
    rand::srand((macroquad::miniquad::date::now() * 1000.0) as u64);

    let sfx = Sfx::load().await;
    let mut game = Game::new();

    loop {
        // Giới hạn dt tối đa để tránh lỗi xuyên vật thể khi cửa sổ bị khựng (lag spike)
        let dt = get_frame_time().min(0.05);

        game.update(dt);
        for event in game.take_events() {
            sfx.play(event);
        }
        game.draw();

        next_frame().await
    }
}
