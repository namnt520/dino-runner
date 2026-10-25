use macroquad::prelude::*;

use crate::config::*;
use crate::theme::Theme;

// ==========================================
// HIỆU ỨNG HÌNH ẢNH: BỤI KHI TIẾP ĐẤT & RUNG MÀN HÌNH
// ==========================================
const SHAKE_DURATION: f32 = 0.35;
const SHAKE_MAGNITUDE: f32 = 6.0;
const DUST_COUNT: usize = 6;
const DUST_GRAVITY: f32 = 500.0;

/// Camera dịch theo độ rung `offset`, giữ nguyên hệ tọa độ của màn hình
/// (gốc ở góc trên trái, trục y hướng xuống).
pub fn shake_camera(offset: Vec2) -> Camera2D {
    let mut camera = Camera2D::from_display_rect(Rect::new(
        offset.x,
        offset.y,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
    ));
    // `from_display_rect` của macroquad 0.4 cho hình bị lật dọc, nên phải lật trục y lại
    camera.zoom.y = -camera.zoom.y;
    camera
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    size: f32,
}

pub struct Effects {
    particles: Vec<Particle>,
    shake_timer: f32,
    shake_offset: Vec2,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            shake_timer: 0.0,
            shake_offset: Vec2::ZERO,
        }
    }

    pub fn reset(&mut self) {
        self.particles.clear();
        self.shake_timer = 0.0;
        self.shake_offset = Vec2::ZERO;
    }

    /// Tung bụi tại vị trí chân (x, y) khi khủng long tiếp đất
    pub fn spawn_dust(&mut self, x: f32, y: f32) {
        for _ in 0..DUST_COUNT {
            let max_life = rand::gen_range(0.3, 0.5);
            self.particles.push(Particle {
                x: x + rand::gen_range(-6.0, 6.0),
                y,
                vx: rand::gen_range(-90.0, 30.0),
                vy: rand::gen_range(-120.0, -40.0),
                life: max_life,
                max_life,
                size: rand::gen_range(2.0, 4.0),
            });
        }
    }

    pub fn start_shake(&mut self) {
        self.shake_timer = SHAKE_DURATION;
    }

    /// Độ lệch camera hiện tại do rung màn hình (bằng 0 khi không rung)
    pub fn shake_offset(&self) -> Vec2 {
        self.shake_offset
    }

    /// `world_speed` là tốc độ cuộn của mặt đất (0 khi game dừng), để bụi trôi theo mặt đất
    pub fn update(&mut self, dt: f32, world_speed: f32) {
        for p in &mut self.particles {
            p.vy += DUST_GRAVITY * dt;
            p.x += (p.vx - world_speed) * dt;
            p.y += p.vy * dt;
            if p.y > GROUND_Y {
                p.y = GROUND_Y;
                p.vy = 0.0;
            }
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        self.shake_timer = (self.shake_timer - dt).max(0.0);
        self.shake_offset = if self.shake_timer > 0.0 {
            // Rung mạnh lúc đầu rồi yếu dần
            let strength = SHAKE_MAGNITUDE * self.shake_timer / SHAKE_DURATION;
            vec2(rand::gen_range(-1.0, 1.0), rand::gen_range(-1.0, 1.0)) * strength
        } else {
            Vec2::ZERO
        };
    }

    pub fn draw(&self, theme: &Theme) {
        let c = theme.primary;
        for p in &self.particles {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            draw_rectangle(p.x, p.y - p.size, p.size, p.size, Color::new(c.r, c.g, c.b, alpha));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chiếu một điểm trong thế giới game ra tọa độ chuẩn hóa của màn hình
    /// (x: -1 trái → 1 phải, y: 1 trên → -1 dưới)
    fn project(camera: &Camera2D, x: f32, y: f32) -> Vec2 {
        let p = camera.matrix().project_point3(vec3(x, y, 0.0));
        vec2(p.x, p.y)
    }

    fn near(a: Vec2, b: Vec2) -> bool {
        (a - b).abs().max_element() < 1e-4
    }

    #[test]
    fn shake_camera_keeps_the_screen_upright() {
        let camera = shake_camera(Vec2::ZERO);
        // Góc trên trái của game phải nằm ở góc trên trái của màn hình
        assert!(near(project(&camera, 0.0, 0.0), vec2(-1.0, 1.0)));
        // Góc dưới phải của game phải nằm ở góc dưới phải của màn hình
        assert!(near(project(&camera, SCREEN_WIDTH, SCREEN_HEIGHT), vec2(1.0, -1.0)));
        // Mặt đất (y lớn) phải ở thấp hơn đỉnh màn hình (y = 0)
        assert!(project(&camera, 0.0, GROUND_Y).y < project(&camera, 0.0, 0.0).y);
    }

    #[test]
    fn shake_camera_shifts_the_view_by_the_offset() {
        let camera = shake_camera(vec2(10.0, -20.0));
        // Camera nhìn lệch sang phải 10px và lên trên 20px, nên điểm (0, 0) trôi sang trái và xuống dưới
        let expected = vec2(-1.0 - 2.0 * 10.0 / SCREEN_WIDTH, 1.0 + 2.0 * -20.0 / SCREEN_HEIGHT);
        assert!(near(project(&camera, 0.0, 0.0), expected));
    }

    #[test]
    fn spawn_dust_adds_particles_at_the_feet() {
        let mut effects = Effects::new();
        effects.spawn_dust(100.0, GROUND_Y);
        assert_eq!(effects.particles.len(), DUST_COUNT);
        for p in &effects.particles {
            assert!(p.x >= 94.0 && p.x <= 106.0);
            assert_eq!(p.y, GROUND_Y);
            assert!(p.vy < 0.0, "bụi phải bay lên lúc đầu");
            assert!(p.life > 0.0 && p.life == p.max_life);
        }
    }

    #[test]
    fn dust_drifts_with_the_scrolling_ground() {
        let mut effects = Effects::new();
        effects.spawn_dust(300.0, GROUND_Y);
        let before: Vec<f32> = effects.particles.iter().map(|p| p.x).collect();

        effects.update(0.05, 400.0);
        for (p, x0) in effects.particles.iter().zip(before) {
            // Trôi sang trái thêm khoảng 400 * 0.05 = 20px so với lúc chỉ có vận tốc riêng
            assert!(p.x < x0 - 10.0);
        }
    }

    #[test]
    fn dust_does_not_move_with_the_world_when_the_game_is_frozen() {
        let mut moving = Effects::new();
        let mut frozen = Effects::new();
        moving.spawn_dust(300.0, GROUND_Y);
        frozen.particles.push(Particle {
            x: 300.0,
            y: GROUND_Y,
            vx: 0.0,
            vy: 0.0,
            life: 0.5,
            max_life: 0.5,
            size: 3.0,
        });
        frozen.update(0.1, 0.0);
        assert_eq!(frozen.particles[0].x, 300.0);
        assert_eq!(moving.particles.len(), DUST_COUNT);
    }

    #[test]
    fn dust_never_sinks_below_the_ground() {
        let mut effects = Effects::new();
        effects.spawn_dust(100.0, GROUND_Y);
        for _ in 0..20 {
            effects.update(1.0 / 60.0, 360.0);
            for p in &effects.particles {
                assert!(p.y <= GROUND_Y);
            }
        }
    }

    #[test]
    fn dust_disappears_after_its_lifetime() {
        let mut effects = Effects::new();
        effects.spawn_dust(100.0, GROUND_Y);
        effects.update(1.0, 360.0);
        assert!(effects.particles.is_empty());
    }

    #[test]
    fn no_shake_by_default() {
        let mut effects = Effects::new();
        effects.update(0.016, 0.0);
        assert_eq!(effects.shake_offset(), Vec2::ZERO);
    }

    #[test]
    fn shake_offset_stays_within_the_magnitude() {
        let mut effects = Effects::new();
        effects.start_shake();
        for _ in 0..20 {
            effects.update(0.01, 0.0);
            let o = effects.shake_offset();
            assert!(o.x.abs() <= SHAKE_MAGNITUDE && o.y.abs() <= SHAKE_MAGNITUDE);
        }
    }

    #[test]
    fn shake_fades_out_completely() {
        let mut effects = Effects::new();
        effects.start_shake();
        effects.update(SHAKE_DURATION + 0.01, 0.0);
        assert_eq!(effects.shake_offset(), Vec2::ZERO);
    }

    #[test]
    fn shake_gets_weaker_over_time() {
        let mut effects = Effects::new();
        effects.start_shake();
        effects.update(0.01, 0.0);
        let early_bound = SHAKE_MAGNITUDE * effects.shake_timer / SHAKE_DURATION;
        effects.update(0.3, 0.0);
        let late_bound = SHAKE_MAGNITUDE * effects.shake_timer / SHAKE_DURATION;
        assert!(late_bound < early_bound);
        let o = effects.shake_offset();
        assert!(o.x.abs() <= late_bound && o.y.abs() <= late_bound);
    }

    #[test]
    fn reset_clears_everything() {
        let mut effects = Effects::new();
        effects.spawn_dust(100.0, GROUND_Y);
        effects.start_shake();
        effects.update(0.01, 0.0);
        effects.reset();
        assert!(effects.particles.is_empty());
        assert_eq!(effects.shake_offset(), Vec2::ZERO);
        effects.update(0.01, 0.0);
        assert_eq!(effects.shake_offset(), Vec2::ZERO);
    }
}
