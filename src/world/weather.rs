use macroquad::prelude::*;

use crate::config::*;
use crate::theme::Theme;

// ==========================================
// THỜI TIẾT: MƯA, GIÓ, BÃO CÁT
// ==========================================
/// Thời gian trời quang giữa hai đợt thời tiết (giây): ngẫu nhiên trong khoảng này
const CALM_SECONDS: (f32, f32) = (20.0, 40.0);
/// Thời gian một đợt thời tiết ở cường độ tối đa (giây)
const ACTIVE_SECONDS: (f32, f32) = (15.0, 30.0);
/// Thời gian thời tiết mạnh dần lên và yếu dần đi (giây)
const FADE_SECONDS: f32 = 3.0;
const MAX_PARTICLES: usize = 500;

/// Gió làm cú nhảy yếu đi tối đa bấy nhiêu (6% = thấp hơn khoảng 12px). Cú nhảy yếu hơn cũng
/// ngắn hơn, nên khoảng cách tối thiểu giữa các chướng ngại (`spawner.rs`) vẫn luôn đủ.
const WIND_JUMP_PENALTY: f32 = 0.06;
const SANDSTORM_JUMP_PENALTY: f32 = 0.04;

const SAND_COLOR: (f32, f32, f32) = (0.76, 0.63, 0.43);

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WeatherKind {
    Clear,
    Rain,
    Wind,
    Sandstorm,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    /// Trời quang, đếm ngược đến đợt thời tiết kế tiếp
    Calm,
    FadeIn,
    Hold,
    FadeOut,
}

impl WeatherKind {
    fn random_active() -> Self {
        match rand::gen_range(0, 3) {
            0 => WeatherKind::Rain,
            1 => WeatherKind::Wind,
            _ => WeatherKind::Sandstorm,
        }
    }

    /// Số hạt sinh ra mỗi giây ở cường độ tối đa
    fn spawn_rate(self) -> f32 {
        match self {
            WeatherKind::Clear => 0.0,
            WeatherKind::Rain => 220.0,
            WeatherKind::Wind => 14.0,
            WeatherKind::Sandstorm => 160.0,
        }
    }

    fn jump_penalty(self) -> f32 {
        match self {
            WeatherKind::Wind => WIND_JUMP_PENALTY,
            WeatherKind::Sandstorm => SANDSTORM_JUMP_PENALTY,
            WeatherKind::Clear | WeatherKind::Rain => 0.0,
        }
    }

    fn label(self) -> Option<&'static str> {
        match self {
            WeatherKind::Clear => None,
            WeatherKind::Rain => Some("RAIN"),
            WeatherKind::Wind => Some("WIND"),
            WeatherKind::Sandstorm => Some("SANDSTORM"),
        }
    }
}

/// Một hạt thời tiết: giọt mưa hoặc vệt gió / hạt cát bay ngang
struct Particle {
    kind: WeatherKind,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    /// Độ dài vệt (chỉ dùng cho gió và cát)
    len: f32,
    /// Giọt mưa biến mất khi rơi tới độ cao này
    kill_y: f32,
}

impl Particle {
    fn spawn(kind: WeatherKind) -> Option<Self> {
        match kind {
            WeatherKind::Clear => None,
            WeatherKind::Rain => Some(Self {
                kind,
                x: rand::gen_range(0.0, SCREEN_WIDTH + 150.0),
                y: -10.0,
                vx: -rand::gen_range(100.0, 180.0),
                vy: rand::gen_range(650.0, 850.0),
                len: 0.0,
                kill_y: rand::gen_range(GROUND_Y, SCREEN_HEIGHT - 5.0),
            }),
            WeatherKind::Wind => Some(Self {
                kind,
                x: SCREEN_WIDTH + 10.0,
                y: rand::gen_range(20.0, GROUND_Y + 60.0),
                vx: -rand::gen_range(500.0, 900.0),
                vy: rand::gen_range(-10.0, 10.0),
                len: rand::gen_range(20.0, 50.0),
                kill_y: 0.0,
            }),
            WeatherKind::Sandstorm => Some(Self {
                kind,
                x: SCREEN_WIDTH + 10.0,
                y: rand::gen_range(30.0, SCREEN_HEIGHT - 10.0),
                vx: -rand::gen_range(600.0, 1100.0),
                vy: rand::gen_range(-25.0, 25.0),
                len: rand::gen_range(6.0, 14.0),
                kill_y: 0.0,
            }),
        }
    }

    fn is_dead(&self) -> bool {
        match self.kind {
            WeatherKind::Rain => self.y > self.kill_y,
            _ => self.x + self.len < -10.0,
        }
    }
}

pub struct Weather {
    kind: WeatherKind,
    phase: Phase,
    /// Thời gian còn lại của pha hiện tại (giây)
    phase_left: f32,
    /// Cường độ 0..1 (mạnh dần lên rồi yếu dần đi)
    intensity: f32,
    spawn_accumulator: f32,
    particles: Vec<Particle>,
}

fn random_between((low, high): (f32, f32)) -> f32 {
    rand::gen_range(low, high)
}

impl Weather {
    pub fn new() -> Self {
        Self {
            kind: WeatherKind::Clear,
            phase: Phase::Calm,
            phase_left: random_between(CALM_SECONDS),
            intensity: 0.0,
            spawn_accumulator: 0.0,
            particles: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Weather::new();
    }

    pub fn kind(&self) -> WeatherKind {
        self.kind
    }

    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    /// Tên thời tiết hiện tại để hiển thị (None khi trời quang hoặc mới bắt đầu yếu đi)
    pub fn label(&self) -> Option<&'static str> {
        if self.intensity > 0.05 {
            self.kind.label()
        } else {
            None
        }
    }

    /// Hệ số lực nhảy: 1.0 khi bình thường, nhỏ hơn một chút khi có gió hoặc bão cát
    pub fn jump_multiplier(&self) -> f32 {
        1.0 - self.kind.jump_penalty() * self.intensity
    }

    /// Thời tiết chỉ bắt đầu sau khi đạt `WEATHER_MIN_SCORE` điểm
    pub fn update(&mut self, dt: f32, score: u32) {
        self.advance_phase(dt, score);
        self.spawn_particles(dt);
        self.move_particles(dt);
    }

    fn advance_phase(&mut self, dt: f32, score: u32) {
        match self.phase {
            Phase::Calm => {
                self.intensity = 0.0;
                if score >= WEATHER_MIN_SCORE {
                    self.phase_left -= dt;
                    if self.phase_left <= 0.0 {
                        self.kind = WeatherKind::random_active();
                        self.phase = Phase::FadeIn;
                        self.phase_left = FADE_SECONDS;
                    }
                }
            }
            Phase::FadeIn => {
                self.phase_left -= dt;
                self.intensity = (1.0 - self.phase_left / FADE_SECONDS).clamp(0.0, 1.0);
                if self.phase_left <= 0.0 {
                    self.phase = Phase::Hold;
                    self.phase_left = random_between(ACTIVE_SECONDS);
                    self.intensity = 1.0;
                }
            }
            Phase::Hold => {
                self.phase_left -= dt;
                if self.phase_left <= 0.0 {
                    self.phase = Phase::FadeOut;
                    self.phase_left = FADE_SECONDS;
                }
            }
            Phase::FadeOut => {
                self.phase_left -= dt;
                self.intensity = (self.phase_left / FADE_SECONDS).clamp(0.0, 1.0);
                if self.phase_left <= 0.0 {
                    self.phase = Phase::Calm;
                    self.kind = WeatherKind::Clear;
                    self.intensity = 0.0;
                    self.phase_left = random_between(CALM_SECONDS);
                }
            }
        }
    }

    fn spawn_particles(&mut self, dt: f32) {
        let rate = self.kind.spawn_rate() * self.intensity;
        if rate <= 0.0 {
            self.spawn_accumulator = 0.0;
            return;
        }
        self.spawn_accumulator += rate * dt;
        while self.spawn_accumulator >= 1.0 {
            self.spawn_accumulator -= 1.0;
            if self.particles.len() < MAX_PARTICLES {
                if let Some(particle) = Particle::spawn(self.kind) {
                    self.particles.push(particle);
                }
            }
        }
    }

    fn move_particles(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| !p.is_dead());
    }

    pub fn draw(&self, theme: &Theme) {
        let c = theme.primary;
        let sand = Color::new(SAND_COLOR.0, SAND_COLOR.1, SAND_COLOR.2, 0.65);

        for p in &self.particles {
            match p.kind {
                WeatherKind::Rain => {
                    let color = Color::new(c.r, c.g, c.b, 0.45);
                    draw_line(p.x, p.y, p.x - p.vx * 0.025, p.y - p.vy * 0.025, 1.0, color);
                }
                WeatherKind::Wind => {
                    let color = Color::new(c.r, c.g, c.b, 0.28);
                    draw_line(p.x, p.y, p.x + p.len, p.y - p.vy * 0.02, 1.0, color);
                }
                WeatherKind::Sandstorm => {
                    draw_line(p.x, p.y, p.x + p.len, p.y - p.vy * 0.01, 1.5, sand);
                }
                WeatherKind::Clear => {}
            }
        }

        // Lớp phủ toàn màn hình: bão cát làm mờ tầm nhìn, mưa làm trời âm u hơn
        let overlay = match self.kind {
            WeatherKind::Sandstorm => Some(Color::new(
                SAND_COLOR.0,
                SAND_COLOR.1,
                SAND_COLOR.2,
                0.28 * self.intensity,
            )),
            WeatherKind::Rain => Some(Color::new(0.05, 0.07, 0.15, 0.10 * self.intensity)),
            WeatherKind::Wind | WeatherKind::Clear => None,
        };
        if let Some(color) = overlay {
            draw_rectangle(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    /// Thời tiết đang ở giữa đợt (cường độ tối đa) với loại cho trước
    fn active(kind: WeatherKind) -> Weather {
        let mut weather = Weather::new();
        weather.kind = kind;
        weather.phase = Phase::Hold;
        weather.phase_left = 1_000.0;
        weather.intensity = 1.0;
        weather
    }

    #[test]
    fn starts_calm_and_clear() {
        let weather = Weather::new();
        assert_eq!(weather.kind, WeatherKind::Clear);
        assert_eq!(weather.phase, Phase::Calm);
        assert_eq!(weather.intensity(), 0.0);
        assert_eq!(weather.jump_multiplier(), 1.0);
        assert_eq!(weather.label(), None);
        assert!(weather.phase_left >= CALM_SECONDS.0 && weather.phase_left <= CALM_SECONDS.1);
    }

    #[test]
    fn stays_calm_below_the_minimum_score() {
        let mut weather = Weather::new();
        weather.phase_left = 0.1;
        weather.update(100.0, WEATHER_MIN_SCORE - 1);
        assert_eq!(weather.phase, Phase::Calm);
        assert_eq!(weather.kind, WeatherKind::Clear);
    }

    #[test]
    fn weather_starts_once_the_calm_timer_runs_out_after_the_minimum_score() {
        let mut weather = Weather::new();
        weather.phase_left = 0.1;
        weather.update(0.2, WEATHER_MIN_SCORE);
        assert_eq!(weather.phase, Phase::FadeIn);
        assert_ne!(weather.kind, WeatherKind::Clear);
        assert!(approx(weather.phase_left, FADE_SECONDS));
    }

    #[test]
    fn fade_in_raises_intensity_gradually_then_holds_at_full() {
        let mut weather = Weather::new();
        weather.phase_left = 0.1;
        weather.update(0.2, WEATHER_MIN_SCORE);

        weather.update(FADE_SECONDS / 2.0, WEATHER_MIN_SCORE);
        assert!(approx(weather.intensity(), 0.5));

        weather.update(FADE_SECONDS, WEATHER_MIN_SCORE);
        assert_eq!(weather.phase, Phase::Hold);
        assert!(approx(weather.intensity(), 1.0));
        assert!(weather.phase_left >= ACTIVE_SECONDS.0 && weather.phase_left <= ACTIVE_SECONDS.1);
    }

    #[test]
    fn weather_fades_out_and_returns_to_clear_skies() {
        let mut weather = active(WeatherKind::Rain);
        weather.phase_left = 0.5;

        weather.update(0.6, WEATHER_MIN_SCORE);
        assert_eq!(weather.phase, Phase::FadeOut);

        weather.update(FADE_SECONDS / 2.0, WEATHER_MIN_SCORE);
        assert!(approx(weather.intensity(), 0.5));

        weather.update(FADE_SECONDS, WEATHER_MIN_SCORE);
        assert_eq!(weather.phase, Phase::Calm);
        assert_eq!(weather.kind, WeatherKind::Clear);
        assert_eq!(weather.intensity(), 0.0);
    }

    #[test]
    fn many_cycles_keep_invariants_and_show_every_kind_of_weather() {
        let mut weather = Weather::new();
        let (mut rain, mut wind, mut sand) = (false, false, false);

        for _ in 0..30_000 {
            weather.update(0.1, 10_000);
            assert!((0.0..=1.0).contains(&weather.intensity()));
            assert!(weather.particles.len() <= MAX_PARTICLES);
            if weather.kind == WeatherKind::Clear {
                assert_eq!(weather.intensity(), 0.0);
            }
            match weather.kind {
                WeatherKind::Rain => rain = true,
                WeatherKind::Wind => wind = true,
                WeatherKind::Sandstorm => sand = true,
                WeatherKind::Clear => {}
            }
        }
        assert!(rain && wind && sand, "rain={rain}, wind={wind}, sand={sand}");
    }

    #[test]
    fn jump_multiplier_only_drops_in_wind_and_sandstorm() {
        assert!(approx(active(WeatherKind::Wind).jump_multiplier(), 1.0 - WIND_JUMP_PENALTY));
        assert!(approx(
            active(WeatherKind::Sandstorm).jump_multiplier(),
            1.0 - SANDSTORM_JUMP_PENALTY
        ));
        assert_eq!(active(WeatherKind::Rain).jump_multiplier(), 1.0);
        assert_eq!(active(WeatherKind::Clear).jump_multiplier(), 1.0);
    }

    #[test]
    fn jump_penalty_scales_with_intensity_and_stays_small() {
        let mut weather = active(WeatherKind::Wind);
        weather.intensity = 0.5;
        assert!(approx(weather.jump_multiplier(), 1.0 - WIND_JUMP_PENALTY / 2.0));
        for kind in [WeatherKind::Wind, WeatherKind::Sandstorm] {
            assert!(active(kind).jump_multiplier() >= 0.9);
        }
    }

    #[test]
    fn label_shows_the_active_weather_only() {
        assert_eq!(active(WeatherKind::Rain).label(), Some("RAIN"));
        assert_eq!(active(WeatherKind::Wind).label(), Some("WIND"));
        assert_eq!(active(WeatherKind::Sandstorm).label(), Some("SANDSTORM"));
        assert_eq!(active(WeatherKind::Clear).label(), None);

        let mut fading = active(WeatherKind::Rain);
        fading.intensity = 0.02;
        assert_eq!(fading.label(), None);
    }

    #[test]
    fn rain_spawns_falling_drops_at_the_expected_rate() {
        let mut weather = active(WeatherKind::Rain);
        // 6 bước x 0.05 giây = 0.3 giây: chưa giọt nào kịp rơi chạm đất, 220 hạt/giây ⇒ khoảng 66 hạt
        for _ in 0..6 {
            weather.update(0.05, WEATHER_MIN_SCORE);
        }
        assert!((64..=68).contains(&weather.particles.len()), "{}", weather.particles.len());
        for p in &weather.particles {
            assert!(p.vy > 0.0, "giọt mưa phải rơi xuống");
            assert!(p.vx < 0.0, "mưa hơi xiên sang trái");
            assert!(p.y > -10.0, "giọt mưa đã rơi xuống từ chỗ sinh ra");
        }
    }

    #[test]
    fn wind_and_sand_enter_from_the_right_and_fly_left() {
        for kind in [WeatherKind::Wind, WeatherKind::Sandstorm] {
            let mut weather = active(kind);
            weather.update(0.5, WEATHER_MIN_SCORE);
            assert!(!weather.particles.is_empty(), "{kind:?}");
            for p in &weather.particles {
                assert!(p.vx < 0.0);
                assert!(p.x < SCREEN_WIDTH + 10.0, "hạt phải đã bay vào màn hình");
                assert!(p.len > 0.0);
            }
        }
    }

    #[test]
    fn sandstorm_is_denser_than_wind() {
        let mut wind = active(WeatherKind::Wind);
        let mut sand = active(WeatherKind::Sandstorm);
        wind.update(0.5, WEATHER_MIN_SCORE);
        sand.update(0.5, WEATHER_MIN_SCORE);
        assert!(sand.particles.len() > wind.particles.len());
    }

    #[test]
    fn particles_disappear_once_the_weather_is_gone() {
        for kind in [WeatherKind::Rain, WeatherKind::Wind, WeatherKind::Sandstorm] {
            let mut weather = active(kind);
            weather.update(0.5, WEATHER_MIN_SCORE);
            assert!(!weather.particles.is_empty());

            // Thời tiết kết thúc: không sinh hạt mới, hạt cũ bay ra khỏi màn hình
            weather.kind = WeatherKind::Clear;
            weather.phase = Phase::Calm;
            weather.intensity = 0.0;
            weather.update(5.0, WEATHER_MIN_SCORE);
            assert!(weather.particles.is_empty(), "{kind:?}");
        }
    }

    #[test]
    fn particle_count_is_capped() {
        let mut weather = active(WeatherKind::Rain);
        weather.spawn_particles(10.0); // yêu cầu 2200 hạt
        assert_eq!(weather.particles.len(), MAX_PARTICLES);
    }

    #[test]
    fn reset_clears_everything() {
        let mut weather = active(WeatherKind::Sandstorm);
        weather.update(0.5, WEATHER_MIN_SCORE);
        weather.reset();
        assert_eq!(weather.kind, WeatherKind::Clear);
        assert_eq!(weather.phase, Phase::Calm);
        assert_eq!(weather.intensity(), 0.0);
        assert!(weather.particles.is_empty());
    }
}
