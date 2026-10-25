use macroquad::prelude::*;

use crate::config::*;
use crate::theme::Theme;

// ==========================================
// ĐỊA HÌNH MẶT ĐẤT (ĐƯỜNG CHÂN TRỜI + SỎI ĐÁ)
// ==========================================
struct GroundParticle {
    x: f32,
    y: f32,
    width: f32,
}

pub struct Ground {
    particles: Vec<GroundParticle>,
}

impl Ground {
    pub fn new() -> Self {
        let mut particles = Vec::new();
        for i in 0..45 {
            particles.push(GroundParticle {
                x: i as f32 * 22.0 + rand::gen_range(0.0, 10.0),
                y: GROUND_Y + rand::gen_range(5.0, 18.0),
                width: rand::gen_range(2.0, 8.0),
            });
        }
        Self { particles }
    }

    /// Tạo lại toàn bộ sỏi đá như lúc bắt đầu
    pub fn reset(&mut self) {
        *self = Ground::new();
    }

    /// Cuộn hiệu ứng sỏi đá trên mặt đất
    pub fn update(&mut self, dt: f32, speed: f32) {
        for p in &mut self.particles {
            p.x -= speed * dt;
            if p.x + p.width < 0.0 {
                p.x = SCREEN_WIDTH + rand::gen_range(5.0, 40.0);
                p.y = GROUND_Y + rand::gen_range(5.0, 18.0);
                p.width = rand::gen_range(2.0, 8.0);
            }
        }
    }

    pub fn draw(&self, theme: &Theme) {
        // Đường chân trời & chi tiết sỏi đá
        draw_line(0.0, GROUND_Y, SCREEN_WIDTH, GROUND_Y, 2.0, theme.primary);
        for p in &self.particles {
            draw_rectangle(p.x, p.y, p.width, 2.0, theme.primary);
        }
    }
}
