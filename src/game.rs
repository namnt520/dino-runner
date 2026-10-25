use macroquad::prelude::*;

use crate::collision;
use crate::config::*;
use crate::effects::{shake_camera, Effects};
use crate::entities::dino::Dino;
use crate::entities::obstacle::Obstacle;
use crate::entities::skin::Skin;
use crate::input::Input;
use crate::score::Score;
use crate::spawner::Spawner;
use crate::theme::Theme;
use crate::ui;
use crate::world::cloud::Clouds;
use crate::world::ground::Ground;
use crate::world::sky::{night_amount, Sky};
use crate::world::weather::{Weather, WeatherKind};

/// Điểm số nhấp nháy bấy nhiêu giây mỗi khi qua mốc 100 điểm
const MILESTONE_FLASH_SECONDS: f32 = 0.8;

// ==========================================
// TRẠNG THÁI GAME (STATE MACHINE)
// ==========================================
#[derive(PartialEq, Clone, Copy)]
pub enum GameState {
    Waiting,
    Playing,
    GameOver,
}

/// Sự kiện xảy ra trong game. `Game` chỉ phát sự kiện, còn việc phát âm thanh
/// do bên ngoài (`main.rs` + `audio.rs`) quyết định, nên logic game không phụ thuộc thiết bị âm thanh.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameEvent {
    Jump,
    Land,
    Die,
    Milestone,
}

// ==========================================
// QUẢN LÝ TỔNG THỂ TRÒ CHƠI (GAME ENGINE)
// ==========================================
pub struct Game {
    state: GameState,
    dino: Dino,
    obstacles: Vec<Obstacle>,
    ground: Ground,
    clouds: Clouds,
    sky: Sky,
    effects: Effects,
    weather: Weather,
    spawner: Spawner,
    score: Score,
    /// Skin đang chọn (đổi bằng ← → ở màn hình chờ và màn hình thua)
    skin: Skin,
    /// Skin vừa mở khóa nhờ ván vừa rồi (để thông báo ở màn hình thua)
    new_skin: Option<Skin>,
    speed: f32,
    /// Mức đêm hiện tại (0 = ngày, 1 = đêm), tính từ điểm số
    night: f32,
    theme: Theme,
    /// Thời gian còn lại của hiệu ứng nhấp nháy điểm số
    flash_timer: f32,
    events: Vec<GameEvent>,
}

impl Game {
    pub fn new() -> Self {
        // Thứ tự khởi tạo (mặt đất rồi mây) giữ nguyên để chuỗi random không đổi
        let ground = Ground::new();
        let clouds = Clouds::new();
        let score = Score::new();
        let skin = Skin::restore(score.skin_index(), score.high());

        Self {
            state: GameState::Waiting,
            dino: Dino::new(),
            obstacles: Vec::new(),
            ground,
            clouds,
            sky: Sky::new(),
            effects: Effects::new(),
            weather: Weather::new(),
            spawner: Spawner::new(),
            score,
            skin,
            new_skin: None,
            speed: INITIAL_SPEED,
            night: 0.0,
            theme: Theme::chrome(),
            flash_timer: 0.0,
            events: Vec::new(),
        }
    }

    /// Lấy (và xóa) các sự kiện đã xảy ra từ lần gọi trước
    pub fn take_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    /// Thời tiết dùng để chọn âm nền (loại, cường độ). Chỉ có tiếng khi đang chơi:
    /// ở màn hình chờ và khi thua thì trả về trời quang để âm nền tắt.
    pub fn ambience(&self) -> (WeatherKind, f32) {
        if self.state == GameState::Playing {
            (self.weather.kind(), self.weather.intensity())
        } else {
            (WeatherKind::Clear, 0.0)
        }
    }

    fn reset(&mut self) {
        self.dino.reset();
        self.obstacles.clear();
        self.ground.reset();
        self.clouds.reset();
        self.effects.reset();
        self.weather.reset();
        self.speed = INITIAL_SPEED;
        self.score.reset();
        self.flash_timer = 0.0;
        self.new_skin = None;
        self.spawner.reset();
        self.state = GameState::Playing;
    }

    pub fn update(&mut self, dt: f32) {
        let input = Input::poll();

        match self.state {
            GameState::Waiting => self.update_waiting(&input),
            GameState::Playing => self.update_playing(dt, &input),
            GameState::GameOver => self.update_game_over(&input),
        }

        // Hiệu ứng chạy ở mọi trạng thái (rung màn hình sau khi chết vẫn tiếp diễn);
        // khi game dừng thì bụi không còn trôi theo mặt đất
        let world_speed = if self.state == GameState::Playing {
            self.speed
        } else {
            0.0
        };
        self.effects.update(dt, world_speed);

        // Ngày / đêm theo điểm số
        self.night = night_amount(self.score.exact());
        self.theme = Theme::chrome().lerp(&Theme::night(), self.night);
    }

    /// ← → đổi giữa các skin đã mở khóa và lưu lại lựa chọn
    fn change_skin(&mut self, input: &Input) {
        let high = self.score.high();
        let next = if input.right_pressed {
            self.skin.next_unlocked(high)
        } else if input.left_pressed {
            self.skin.prev_unlocked(high)
        } else {
            return;
        };
        if next != self.skin {
            self.skin = next;
            self.score.set_skin_index(next.index());
        }
    }

    fn update_waiting(&mut self, input: &Input) {
        self.change_skin(input);
        if input.jump_pressed {
            if self.dino.jump_with(self.weather.jump_multiplier()) {
                self.events.push(GameEvent::Jump);
            }
            self.state = GameState::Playing;
        }
    }

    fn update_playing(&mut self, dt: f32, input: &Input) {
        // Xử lý nhảy khi đang chạm đất và người chơi nhấn/giữ phím nhảy
        // (gió và bão cát làm cú nhảy yếu đi một chút)
        let jump_strength = self.weather.jump_multiplier();
        if (input.jump_pressed || input.jump_held) && self.dino.jump_with(jump_strength) {
            self.events.push(GameEvent::Jump);
        }

        let was_airborne = !self.dino.is_grounded();
        self.dino.update(dt, input.down);
        if was_airborne && self.dino.is_grounded() {
            let (foot_x, foot_y) = self.dino.foot_position();
            self.effects.spawn_dust(foot_x, foot_y);
            self.events.push(GameEvent::Land);
        }

        // Tăng tốc độ và điểm số theo thời gian sống
        self.speed = (self.speed + SPEED_ACCELERATION * dt).min(MAX_SPEED);
        self.score.advance(dt, self.speed);

        // Qua mốc 100 điểm: phát tiếng và làm điểm số nhấp nháy
        self.flash_timer = (self.flash_timer - dt).max(0.0);
        if self.score.take_milestone() {
            self.flash_timer = MILESTONE_FLASH_SECONDS;
            self.events.push(GameEvent::Milestone);
        }

        // Thời tiết (mưa, gió, bão cát) bắt đầu khi đủ điểm
        self.weather.update(dt, self.score.current());

        // Cuộn nền
        self.sky.update(dt, self.speed);
        self.ground.update(dt, self.speed);
        self.clouds.update(dt, self.speed);

        // Cập nhật vị trí các chướng ngại vật và xóa khi đi khỏi màn hình
        for obs in &mut self.obstacles {
            obs.update(dt, self.speed);
        }
        self.obstacles.retain(|obs| !obs.is_off_screen());

        // Sinh chướng ngại vật mới
        self.spawner.update(&mut self.obstacles, self.speed, self.score.current());

        // Kiểm tra va chạm
        if collision::dino_hits_any(&self.dino, &self.obstacles) {
            self.state = GameState::GameOver;
            let old_high = self.score.high();
            self.score.commit_high();
            self.new_skin = Skin::newly_unlocked(old_high, self.score.high());
            self.effects.start_shake();
            self.events.push(GameEvent::Die);
        }
    }

    fn update_game_over(&mut self, input: &Input) {
        self.change_skin(input);
        if input.jump_pressed {
            self.reset();
        }
    }

    pub fn draw(&self) {
        clear_background(self.theme.bg);

        // Rung màn hình: dịch camera đi một chút (hệ tọa độ vẫn như toạ độ màn hình)
        let shake = self.effects.shake_offset();
        let shaking = shake != Vec2::ZERO;
        if shaking {
            set_camera(&shake_camera(shake));
        }

        // 1. Sao và mặt trăng (chỉ hiện khi trời tối)
        self.sky.draw(self.night, &self.theme);

        // 2. Mây nền
        self.clouds.draw(&self.theme);

        // 3. Đường chân trời & sỏi đá
        self.ground.draw(&self.theme);

        // 4. Các chướng ngại vật (xương rồng, chim)
        for obs in &self.obstacles {
            obs.draw(&self.theme);
        }

        // 5. Khủng long và bụi tiếp đất
        let is_dead = self.state == GameState::GameOver;
        self.dino.draw(is_dead, &self.theme, self.skin);
        self.effects.draw(&self.theme);

        // 6. Thời tiết (hạt mưa, gió, cát và lớp phủ làm mờ tầm nhìn)
        self.weather.draw(&self.theme);

        // 7. Điểm số & UI theo trạng thái
        ui::draw_hud(&self.score, &self.theme, ui::score_blinks_off(self.flash_timer));
        ui::draw_weather_label(self.weather.label(), self.weather.intensity(), &self.theme);
        ui::draw_state_overlay(self.state, &self.score, &self.theme, self.skin, self.new_skin);

        if shaking {
            set_default_camera();
        }
    }
}
