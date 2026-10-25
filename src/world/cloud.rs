use macroquad::prelude::*;

use crate::config::*;
use crate::theme::Theme;

// ==========================================
// ĐÁM MÂY TRANG TRÍ (PARALLAX)
// ==========================================
struct Cloud {
    x: f32,
    y: f32,
    width: f32,
}

pub struct Clouds {
    clouds: Vec<Cloud>,
}

impl Clouds {
    pub fn new() -> Self {
        let mut clouds = Vec::new();
        for i in 0..4 {
            clouds.push(Cloud {
                x: 120.0 + i as f32 * 210.0,
                y: rand::gen_range(50.0, 150.0),
                width: rand::gen_range(45.0, 75.0),
            });
        }
        Self { clouds }
    }

    /// Tạo lại các đám mây như lúc bắt đầu
    pub fn reset(&mut self) {
        *self = Clouds::new();
    }

    /// Cuộn đám mây (Parallax scrolling chậm hơn mặt đất)
    pub fn update(&mut self, dt: f32, speed: f32) {
        for c in &mut self.clouds {
            c.x -= (speed * 0.22) * dt;
            if c.x + c.width < 0.0 {
                c.x = SCREEN_WIDTH + rand::gen_range(20.0, 100.0);
                c.y = rand::gen_range(50.0, 150.0);
            }
        }
    }

    pub fn draw(&self, theme: &Theme) {
        for c in &self.clouds {
            draw_rectangle(c.x, c.y, c.width, 12.0, theme.cloud);
            draw_rectangle(c.x + 10.0, c.y - 8.0, c.width - 20.0, 8.0, theme.cloud);
        }
    }
}
