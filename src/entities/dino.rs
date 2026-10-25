use macroquad::prelude::*;

use crate::config::*;
use crate::theme::Theme;

/// Chiều cao của khủng long khi cúi
const DUCK_HEIGHT: f32 = 26.0;

// ==========================================
// NHÂN VẬT CHÍNH: KHỦNG LONG (DINO)
// ==========================================
pub struct Dino {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    vy: f32,
    is_grounded: bool,
    is_ducking: bool,
    anim_timer: f32,
    leg_step: bool,
}

impl Dino {
    pub fn new() -> Self {
        let height = 46.0;
        Self {
            x: 80.0,
            y: GROUND_Y - height,
            width: 42.0,
            height,
            vy: 0.0,
            is_grounded: true,
            is_ducking: false,
            anim_timer: 0.0,
            leg_step: false,
        }
    }

    pub fn reset(&mut self) {
        self.y = GROUND_Y - self.height;
        self.vy = 0.0;
        self.is_grounded = true;
        self.is_ducking = false;
        self.anim_timer = 0.0;
    }

    pub fn is_grounded(&self) -> bool {
        self.is_grounded
    }

    pub fn is_ducking(&self) -> bool {
        self.is_ducking
    }

    /// Nhảy nếu đang đứng trên đất. Trả về `true` nếu thực sự vừa nhảy.
    pub fn jump(&mut self) -> bool {
        if self.is_grounded {
            self.vy = JUMP_FORCE;
            self.is_grounded = false;
            self.is_ducking = false;
            true
        } else {
            false
        }
    }

    /// Vị trí bàn chân (giữa thân, chạm đất), dùng để tung bụi khi tiếp đất
    pub fn foot_position(&self) -> (f32, f32) {
        (self.x + 22.0, GROUND_Y)
    }

    /// `down` là trạng thái đang giữ phím ↓:
    /// - trên đất: cúi xuống
    /// - trên không: rơi nhanh hơn
    pub fn update(&mut self, dt: f32, down: bool) {
        if !self.is_grounded {
            self.is_ducking = false;

            // Cho phép rơi nhanh hơn nếu người chơi nhấn mũi tên xuống khi đang nhảy
            let current_gravity = if down && self.vy > -100.0 {
                GRAVITY * FAST_FALL_MULTIPLIER
            } else {
                GRAVITY
            };

            self.vy += current_gravity * dt;
            self.y += self.vy * dt;

            // Kiểm tra chạm đất (Grounded Check)
            if self.y >= GROUND_Y - self.height {
                self.y = GROUND_Y - self.height;
                self.vy = 0.0;
                self.is_grounded = true;
            }
        } else {
            self.is_ducking = down;

            // Hoạt ảnh chạy bộ khi ở trên mặt đất
            self.anim_timer += dt;
            if self.anim_timer >= 0.09 {
                self.anim_timer = 0.0;
                self.leg_step = !self.leg_step;
            }
        }
    }

    pub fn hitbox(&self) -> Rect {
        // Thu gọn Hitbox một chút ở rìa để tránh cảm giác va chạm oan (pixel-forgiving)
        if self.is_ducking {
            // Khi cúi: thấp hơn và dài hơn, đáy vẫn ngang với đáy hitbox lúc đứng
            Rect::new(self.x + 6.0, GROUND_Y - DUCK_HEIGHT + 5.0, 44.0, 17.0)
        } else {
            Rect::new(
                self.x + 8.0,
                self.y + 6.0,
                self.width - 14.0,
                self.height - 10.0,
            )
        }
    }

    pub fn draw(&self, is_dead: bool, theme: &Theme) {
        if self.is_ducking() && self.is_grounded() {
            self.draw_ducking(is_dead, theme);
        } else {
            self.draw_standing(is_dead, theme);
        }
    }

    fn draw_ducking(&self, is_dead: bool, theme: &Theme) {
        let color = theme.primary;
        let bg_color = theme.bg;
        let x = self.x;
        let t = GROUND_Y - DUCK_HEIGHT; // đỉnh của sprite khi cúi

        // 1. Đầu & mõm (vươn ra phía trước)
        draw_rectangle(x + 30.0, t, 24.0, 14.0, color);
        draw_rectangle(x + 34.0, t + 14.0, 18.0, 3.0, color); // Hàm dưới

        // 2. Mắt
        if is_dead {
            draw_rectangle(x + 36.0, t + 1.0, 6.0, 6.0, bg_color);
            draw_rectangle(x + 38.0, t + 3.0, 2.0, 2.0, color);
        } else {
            draw_rectangle(x + 37.0, t + 3.0, 4.0, 4.0, bg_color);
        }

        // 3. Thân nằm ngang
        draw_rectangle(x + 4.0, t + 6.0, 30.0, 14.0, color);

        // 4. Đuôi
        draw_rectangle(x, t + 8.0, 6.0, 6.0, color);

        // 5. Đôi chân (đáy chân luôn chạm đất, chân co lại luân phiên khi chạy)
        let (left_h, right_h) = if is_dead {
            (6.0, 6.0)
        } else if self.leg_step {
            (6.0, 3.0)
        } else {
            (3.0, 6.0)
        };
        draw_rectangle(x + 10.0, GROUND_Y - left_h, 5.0, left_h, color);
        draw_rectangle(x + 22.0, GROUND_Y - right_h, 5.0, right_h, color);
    }

    fn draw_standing(&self, is_dead: bool, theme: &Theme) {
        let color = theme.primary;
        let bg_color = theme.bg;
        let x = self.x;
        let y = self.y;

        // 1. Phần Đầu & Mõm
        draw_rectangle(x + 18.0, y, 24.0, 16.0, color);
        draw_rectangle(x + 22.0, y + 16.0, 18.0, 3.0, color); // Hàm dưới

        // 2. Mắt
        if is_dead {
            // Mắt mở to khi Game Over
            draw_rectangle(x + 23.0, y + 3.0, 6.0, 6.0, bg_color);
            draw_rectangle(x + 25.0, y + 5.0, 2.0, 2.0, color);
        } else {
            draw_rectangle(x + 24.0, y + 4.0, 4.0, 4.0, bg_color);
        }

        // 3. Cổ & Thân chính
        draw_rectangle(x + 14.0, y + 14.0, 14.0, 18.0, color);
        draw_rectangle(x + 6.0, y + 18.0, 18.0, 16.0, color);

        // 4. Đuôi (vát bậc thang)
        draw_rectangle(x, y + 16.0, 6.0, 8.0, color);
        draw_rectangle(x + 3.0, y + 22.0, 5.0, 8.0, color);

        // 5. Tay nhỏ phía trước
        draw_rectangle(x + 28.0, y + 22.0, 6.0, 3.0, color);
        draw_rectangle(x + 32.0, y + 22.0, 2.0, 6.0, color);

        // 6. Đôi chân (thay đổi theo trạng thái nhảy hoặc chạy)
        if !self.is_grounded || is_dead {
            draw_rectangle(x + 10.0, y + 34.0, 5.0, 12.0, color);
            draw_rectangle(x + 20.0, y + 34.0, 5.0, 12.0, color);
        } else if self.leg_step {
            // Bước chân trái, co chân phải
            draw_rectangle(x + 10.0, y + 34.0, 5.0, 12.0, color);
            draw_rectangle(x + 10.0, y + 44.0, 7.0, 2.0, color);
            draw_rectangle(x + 20.0, y + 34.0, 5.0, 7.0, color);
        } else {
            // Co chân trái, bước chân phải
            draw_rectangle(x + 10.0, y + 34.0, 5.0, 7.0, color);
            draw_rectangle(x + 20.0, y + 34.0, 5.0, 12.0, color);
            draw_rectangle(x + 20.0, y + 44.0, 7.0, 2.0, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    /// Số khung hình từ lúc nhảy đến lúc chạm đất
    fn frames_in_air(down: bool) -> u32 {
        let mut dino = Dino::new();
        dino.jump();
        let mut frames = 0;
        while !dino.is_grounded() {
            dino.update(DT, down);
            frames += 1;
            assert!(frames < 10_000, "khủng long không bao giờ chạm đất");
        }
        frames
    }

    #[test]
    fn holding_down_on_the_ground_ducks() {
        let mut dino = Dino::new();
        assert!(!dino.is_ducking());
        dino.update(DT, true);
        assert!(dino.is_ducking());
    }

    #[test]
    fn releasing_down_stands_up_again() {
        let mut dino = Dino::new();
        dino.update(DT, true);
        dino.update(DT, false);
        assert!(!dino.is_ducking());
    }

    #[test]
    fn ducking_hitbox_is_lower_but_keeps_the_same_bottom() {
        let mut dino = Dino::new();
        let standing = dino.hitbox();
        dino.update(DT, true);
        let ducking = dino.hitbox();

        assert!(ducking.y > standing.y, "hitbox khi cúi phải thấp hơn");
        assert!(ducking.h < standing.h);
        assert!(approx(ducking.y + ducking.h, standing.y + standing.h));
    }

    #[test]
    fn jumping_cancels_the_duck() {
        let mut dino = Dino::new();
        dino.update(DT, true);
        dino.jump();
        assert!(!dino.is_ducking());
        assert!(!dino.is_grounded());
    }

    #[test]
    fn holding_down_in_the_air_does_not_duck() {
        let mut dino = Dino::new();
        dino.jump();
        for _ in 0..5 {
            dino.update(DT, true);
            assert!(!dino.is_ducking());
        }
    }

    #[test]
    fn holding_down_in_the_air_makes_the_dino_fall_faster() {
        assert!(frames_in_air(true) < frames_in_air(false));
    }

    #[test]
    fn landing_while_holding_down_ducks_on_the_next_frame() {
        let mut dino = Dino::new();
        dino.jump();
        while !dino.is_grounded() {
            dino.update(DT, true);
        }
        dino.update(DT, true);
        assert!(dino.is_ducking());
    }

    #[test]
    fn reset_clears_the_duck() {
        let mut dino = Dino::new();
        dino.update(DT, true);
        dino.reset();
        assert!(!dino.is_ducking());
        assert!(dino.is_grounded());
    }

    #[test]
    fn jump_reports_whether_it_actually_jumped() {
        let mut dino = Dino::new();
        assert!(dino.jump(), "đứng trên đất thì nhảy được");
        assert!(!dino.jump(), "đang ở trên không thì không nhảy lần hai");
    }

    #[test]
    fn foot_position_is_on_the_ground_in_front_of_the_body() {
        let dino = Dino::new();
        let (x, y) = dino.foot_position();
        assert_eq!(y, GROUND_Y);
        let hitbox = dino.hitbox();
        assert!(x > hitbox.x && x < hitbox.x + hitbox.w);
    }
}
