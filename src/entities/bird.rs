use macroquad::prelude::*;

use crate::config::*;
use crate::theme::Theme;

// ==========================================
// CHƯỚNG NGẠI VẬT: CHIM BAY (PTERODACTYL)
// ==========================================
const BIRD_WIDTH: f32 = 44.0;

/// Độ cao chim bay. Mỗi tầng đòi hỏi một cách né khác nhau:
/// - `High`: bay phía trên đầu, đứng yên hoặc cúi đều qua được (nhảy lên thì có thể đâm vào)
/// - `Mid`: ngang đầu, phải cúi (hoặc nhảy qua)
/// - `Low`: sát đất, phải nhảy (cúi không tránh được)
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BirdLane {
    High,
    Mid,
    Low,
}

impl BirdLane {
    /// Tọa độ y của đỉnh sprite
    fn sprite_y(self) -> f32 {
        match self {
            BirdLane::High => GROUND_Y - 84.0,
            BirdLane::Mid => GROUND_Y - 54.0,
            BirdLane::Low => GROUND_Y - 30.0,
        }
    }

    pub fn random() -> Self {
        match rand::gen_range(0, 3) {
            0 => BirdLane::High,
            1 => BirdLane::Mid,
            _ => BirdLane::Low,
        }
    }
}

pub struct Bird {
    x: f32,
    y: f32,
    anim_timer: f32,
    wing_up: bool,
}

impl Bird {
    /// Sinh chim ở một tầng ngẫu nhiên
    pub fn spawn(start_x: f32) -> Self {
        Self::in_lane(start_x, BirdLane::random())
    }

    pub fn in_lane(start_x: f32, lane: BirdLane) -> Self {
        Self {
            x: start_x,
            y: lane.sprite_y(),
            anim_timer: 0.0,
            wing_up: true,
        }
    }

    /// Chim bay cùng tốc độ với mặt đất, nên khoảng cách giữa các chướng ngại
    /// do `Spawner` đảm bảo không bị thu hẹp lại.
    pub fn update(&mut self, dt: f32, speed: f32) {
        self.x -= speed * dt;

        // Vỗ cánh
        self.anim_timer += dt;
        if self.anim_timer >= 0.12 {
            self.anim_timer = 0.0;
            self.wing_up = !self.wing_up;
        }
    }

    /// Mép phải của chim (dùng để tính khoảng cách spawn)
    pub fn right_edge(&self) -> f32 {
        self.x + BIRD_WIDTH
    }

    /// Chim đã bay khỏi màn hình và có thể xóa
    pub fn is_off_screen(&self) -> bool {
        !(self.right_edge() > -20.0)
    }

    pub fn hitboxes(&self) -> Vec<Rect> {
        // Thu gọn so với sprite (44x28) để tránh va chạm oan ở cánh và đuôi
        vec![Rect::new(self.x + 4.0, self.y + 4.0, 36.0, 20.0)]
    }

    pub fn draw(&self, theme: &Theme) {
        let color = theme.primary;
        let bg_color = theme.bg;
        let x = self.x;
        let y = self.y;

        // 1. Thân
        draw_rectangle(x + 10.0, y + 10.0, 26.0, 8.0, color);

        // 2. Đầu & mỏ (quay sang trái, hướng bay)
        draw_rectangle(x + 3.0, y + 8.0, 10.0, 8.0, color);
        draw_rectangle(x, y + 11.0, 4.0, 3.0, color);

        // 3. Mắt
        draw_rectangle(x + 6.0, y + 10.0, 2.0, 2.0, bg_color);

        // 4. Đuôi
        draw_rectangle(x + 36.0, y + 12.0, 8.0, 4.0, color);

        // 5. Cánh (lên hoặc xuống)
        if self.wing_up {
            draw_rectangle(x + 16.0, y, 8.0, 10.0, color);
            draw_rectangle(x + 24.0, y + 3.0, 6.0, 7.0, color);
        } else {
            draw_rectangle(x + 16.0, y + 18.0, 8.0, 10.0, color);
            draw_rectangle(x + 24.0, y + 18.0, 6.0, 7.0, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    fn hitbox_of(lane: BirdLane) -> Rect {
        Bird::in_lane(500.0, lane).hitboxes()[0]
    }

    #[test]
    fn lanes_are_ordered_from_high_to_low() {
        let high = hitbox_of(BirdLane::High);
        let mid = hitbox_of(BirdLane::Mid);
        let low = hitbox_of(BirdLane::Low);
        assert!(high.y < mid.y && mid.y < low.y);
    }

    #[test]
    fn every_lane_flies_above_the_ground() {
        for lane in [BirdLane::High, BirdLane::Mid, BirdLane::Low] {
            let hb = hitbox_of(lane);
            assert!(hb.y + hb.h < GROUND_Y, "{lane:?} chạm đất");
        }
    }

    #[test]
    fn hitbox_lies_inside_the_sprite() {
        let bird = Bird::in_lane(500.0, BirdLane::Mid);
        let hb = bird.hitboxes()[0];
        assert!(hb.x >= bird.x && hb.x + hb.w <= bird.x + BIRD_WIDTH);
        assert!(hb.y >= bird.y && hb.y + hb.h <= bird.y + 28.0);
    }

    #[test]
    fn update_moves_the_bird_left_by_speed_times_dt() {
        let mut bird = Bird::in_lane(500.0, BirdLane::High);
        bird.update(0.5, 400.0);
        assert!(approx(bird.x, 300.0));
    }

    #[test]
    fn wings_flap_over_time() {
        let mut bird = Bird::in_lane(500.0, BirdLane::High);
        assert!(bird.wing_up);
        bird.update(0.13, 0.0);
        assert!(!bird.wing_up);
        bird.update(0.13, 0.0);
        assert!(bird.wing_up);
    }

    #[test]
    fn right_edge_and_off_screen_threshold() {
        let mut bird = Bird::in_lane(0.0, BirdLane::High);
        assert!(approx(bird.right_edge(), BIRD_WIDTH));
        assert!(!bird.is_off_screen());

        bird.x = -BIRD_WIDTH - 19.0; // mép phải ở -19: vẫn còn trong ngưỡng
        assert!(!bird.is_off_screen());
        bird.x = -BIRD_WIDTH - 20.0; // mép phải ở -20: ra khỏi màn hình
        assert!(bird.is_off_screen());
    }

    #[test]
    fn random_lane_produces_every_lane() {
        let mut seen = [false; 3];
        for _ in 0..300 {
            match BirdLane::random() {
                BirdLane::High => seen[0] = true,
                BirdLane::Mid => seen[1] = true,
                BirdLane::Low => seen[2] = true,
            }
        }
        assert!(seen.iter().all(|s| *s));
    }
}
