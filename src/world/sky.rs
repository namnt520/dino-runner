use macroquad::prelude::*;

use crate::config::*;
use crate::theme::Theme;

// ==========================================
// BẦU TRỜI: NGÀY / ĐÊM, SAO VÀ MẶT TRĂNG
// ==========================================
/// Mỗi pha (ngày hoặc đêm) kéo dài bấy nhiêu điểm
const PHASE_POINTS: f32 = 700.0;
/// Phần cuối của mỗi pha dùng để chuyển dần sang pha kế tiếp
const TRANSITION_POINTS: f32 = 100.0;

const STAR_COUNT: usize = 18;
const MOON_X: f32 = 620.0;
const MOON_Y: f32 = 75.0;

/// Mức "đêm" theo điểm: 0 = ban ngày hoàn toàn, 1 = ban đêm hoàn toàn.
/// Chu kỳ 1400 điểm: ngày 0..600, hoàng hôn 600..700, đêm 700..1300, bình minh 1300..1400.
pub fn night_amount(score: f32) -> f32 {
    let cycle = 2.0 * PHASE_POINTS;
    let t = score.rem_euclid(cycle);
    let dusk_start = PHASE_POINTS - TRANSITION_POINTS;
    let dawn_start = cycle - TRANSITION_POINTS;

    let linear = if t < dusk_start {
        0.0
    } else if t < PHASE_POINTS {
        (t - dusk_start) / TRANSITION_POINTS
    } else if t < dawn_start {
        1.0
    } else {
        1.0 - (t - dawn_start) / TRANSITION_POINTS
    };
    smoothstep(linear)
}

fn smoothstep(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

struct Star {
    x: f32,
    y: f32,
    size: f32,
    phase: f32,
    twinkle_speed: f32,
}

pub struct Sky {
    stars: Vec<Star>,
    time: f32,
}

impl Sky {
    pub fn new() -> Self {
        let mut stars = Vec::with_capacity(STAR_COUNT);
        for _ in 0..STAR_COUNT {
            stars.push(Star {
                x: rand::gen_range(0.0, SCREEN_WIDTH),
                y: rand::gen_range(15.0, 170.0),
                size: rand::gen_range(1.0, 2.5),
                phase: rand::gen_range(0.0, 6.28),
                twinkle_speed: rand::gen_range(1.5, 3.5),
            });
        }
        Self { stars, time: 0.0 }
    }

    /// Sao trôi rất chậm về bên trái (xa hơn mây nên chậm hơn nhiều) và nhấp nháy
    pub fn update(&mut self, dt: f32, speed: f32) {
        self.time += dt;
        for star in &mut self.stars {
            star.x = (star.x - speed * 0.03 * dt).rem_euclid(SCREEN_WIDTH);
        }
    }

    /// Vẽ sao và mặt trăng, mờ dần theo mức đêm `night` (0..1)
    pub fn draw(&self, night: f32, theme: &Theme) {
        if night <= 0.01 {
            return;
        }
        let c = theme.primary;

        for star in &self.stars {
            let twinkle = 0.55 + 0.45 * (self.time * star.twinkle_speed + star.phase).sin();
            let color = Color::new(c.r, c.g, c.b, night * twinkle);
            draw_rectangle(star.x, star.y, star.size, star.size, color);
        }

        // Mặt trăng lưỡi liềm: một hình tròn, khoét bớt bằng hình tròn màu nền
        draw_circle(MOON_X, MOON_Y, 16.0, Color::new(c.r, c.g, c.b, night));
        draw_circle(MOON_X + 7.0, MOON_Y - 4.0, 14.0, theme.bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn it_is_day_at_the_start_and_for_most_of_the_first_phase() {
        assert!(approx(night_amount(0.0), 0.0));
        assert!(approx(night_amount(300.0), 0.0));
        assert!(approx(night_amount(600.0), 0.0));
    }

    #[test]
    fn dusk_blends_from_day_to_night_between_600_and_700() {
        assert!(approx(night_amount(650.0), 0.5));
        assert!(night_amount(620.0) < night_amount(650.0));
        assert!(night_amount(650.0) < night_amount(680.0));
        assert!(approx(night_amount(700.0), 1.0));
    }

    #[test]
    fn it_is_fully_dark_between_700_and_1300() {
        for score in [700.0, 900.0, 1000.0, 1299.0, 1300.0] {
            assert!(approx(night_amount(score), 1.0), "score {score}");
        }
    }

    #[test]
    fn dawn_blends_back_to_day_between_1300_and_1400() {
        assert!(approx(night_amount(1350.0), 0.5));
        assert!(night_amount(1320.0) > night_amount(1350.0));
        assert!(night_amount(1350.0) > night_amount(1380.0));
        assert!(approx(night_amount(1400.0), 0.0));
    }

    #[test]
    fn the_cycle_repeats_every_1400_points() {
        for score in [0.0, 250.0, 650.0, 800.0, 1350.0] {
            assert!(approx(night_amount(score), night_amount(score + 1400.0)));
            assert!(approx(night_amount(score), night_amount(score + 2800.0)));
        }
    }

    #[test]
    fn night_amount_always_stays_between_zero_and_one() {
        let mut score = 0.0;
        while score < 5000.0 {
            let n = night_amount(score);
            assert!((0.0..=1.0).contains(&n), "score {score}: {n}");
            score += 7.3;
        }
    }

    #[test]
    fn transitions_have_no_sudden_jumps() {
        let mut score = 0.0;
        let mut previous = night_amount(0.0);
        while score < 2800.0 {
            score += 1.0;
            let current = night_amount(score);
            assert!((current - previous).abs() < 0.05, "nhảy vọt tại điểm {score}");
            previous = current;
        }
    }

    #[test]
    fn stars_start_inside_the_sky() {
        let sky = Sky::new();
        assert_eq!(sky.stars.len(), STAR_COUNT);
        for star in &sky.stars {
            assert!(star.x >= 0.0 && star.x < SCREEN_WIDTH);
            assert!(star.y >= 15.0 && star.y < GROUND_Y);
        }
    }

    #[test]
    fn stars_drift_left_and_wrap_around_the_screen() {
        let mut sky = Sky::new();
        sky.stars[0].x = 100.0;
        sky.update(1.0, 400.0); // 400 * 0.03 * 1 = 12px
        assert!(approx(sky.stars[0].x, 88.0));

        sky.stars[0].x = 5.0;
        sky.update(1.0, 400.0);
        assert!(approx(sky.stars[0].x, SCREEN_WIDTH - 7.0));
        for star in &sky.stars {
            assert!(star.x >= 0.0 && star.x < SCREEN_WIDTH);
        }
    }
}
