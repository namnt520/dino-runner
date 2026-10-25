use macroquad::audio::{
    load_sound_from_bytes, play_sound, play_sound_once, set_sound_volume, stop_sound,
    PlaySoundParams, Sound,
};

use crate::game::GameEvent;
use crate::synth;
use crate::world::weather::WeatherKind;

/// Âm lượng tối đa của âm nền (đã nhân với mức âm thanh gốc trong `synth.rs`)
const MAX_AMBIENCE_VOLUME: f32 = 0.8;
/// Cường độ thời tiết dưới mức này thì coi như không có tiếng
const AMBIENCE_MIN_INTENSITY: f32 = 0.01;

/// Âm nền nào nên đang phát: không có khi trời quang hoặc thời tiết quá yếu
fn ambience_target(kind: WeatherKind, intensity: f32) -> Option<WeatherKind> {
    if kind == WeatherKind::Clear || intensity < AMBIENCE_MIN_INTENSITY {
        None
    } else {
        Some(kind)
    }
}

/// Âm lượng âm nền theo cường độ thời tiết: nhỏ dần khi thời tiết yếu đi
fn ambience_volume(intensity: f32) -> f32 {
    intensity.clamp(0.0, 1.0) * MAX_AMBIENCE_VOLUME
}

/// Âm thanh của game. Âm thanh được tổng hợp bằng code (`synth.rs`) nên không cần file đi kèm.
/// Nếu tải âm thanh lỗi (ví dụ máy không có thiết bị âm thanh), game vẫn chạy bình thường, chỉ im lặng.
pub struct Sfx {
    jump: Option<Sound>,
    die: Option<Sound>,
    milestone: Option<Sound>,
    rain: Option<Sound>,
    wind: Option<Sound>,
    sandstorm: Option<Sound>,
    /// Âm nền đang phát lặp (nếu có)
    ambience: Option<WeatherKind>,
}

async fn load_sfx(name: &str, wav: Vec<u8>) -> Option<Sound> {
    match load_sound_from_bytes(&wav).await {
        Ok(sound) => Some(sound),
        Err(err) => {
            eprintln!("Không tải được âm thanh '{name}': {err:?}");
            None
        }
    }
}

impl Sfx {
    pub async fn load() -> Self {
        Self {
            jump: load_sfx("jump", synth::jump_sound()).await,
            die: load_sfx("die", synth::die_sound()).await,
            milestone: load_sfx("milestone", synth::milestone_sound()).await,
            rain: load_sfx("rain", synth::rain_sound()).await,
            wind: load_sfx("wind", synth::wind_sound()).await,
            sandstorm: load_sfx("sandstorm", synth::sandstorm_sound()).await,
            ambience: None,
        }
    }

    /// Phát âm thanh tương ứng với sự kiện của game (sự kiện không có âm thì bỏ qua)
    pub fn play(&self, event: GameEvent) {
        let sound = match event {
            GameEvent::Jump => &self.jump,
            GameEvent::Die => &self.die,
            GameEvent::Milestone => &self.milestone,
            GameEvent::Land => return,
        };
        if let Some(sound) = sound {
            play_sound_once(sound);
        }
    }

    fn ambience_loop(&self, kind: WeatherKind) -> Option<&Sound> {
        match kind {
            WeatherKind::Clear => None,
            WeatherKind::Rain => self.rain.as_ref(),
            WeatherKind::Wind => self.wind.as_ref(),
            WeatherKind::Sandstorm => self.sandstorm.as_ref(),
        }
    }

    /// Gọi mỗi khung hình: bật, đổi, chỉnh âm lượng hoặc tắt âm nền theo thời tiết hiện tại
    pub fn update_ambience(&mut self, kind: WeatherKind, intensity: f32) {
        let target = ambience_target(kind, intensity);
        let volume = ambience_volume(intensity);

        if target != self.ambience {
            if let Some(sound) = self.ambience.and_then(|k| self.ambience_loop(k)) {
                stop_sound(sound);
            }
            if let Some(sound) = target.and_then(|k| self.ambience_loop(k)) {
                play_sound(sound, PlaySoundParams { looped: true, volume });
            }
            self.ambience = target;
        } else if let Some(sound) = target.and_then(|k| self.ambience_loop(k)) {
            set_sound_volume(sound, volume);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_weather_has_no_ambience() {
        assert_eq!(ambience_target(WeatherKind::Clear, 1.0), None);
    }

    #[test]
    fn active_weather_plays_its_own_loop() {
        for kind in [WeatherKind::Rain, WeatherKind::Wind, WeatherKind::Sandstorm] {
            assert_eq!(ambience_target(kind, 0.5), Some(kind));
        }
    }

    #[test]
    fn a_vanishing_weather_is_silent() {
        assert_eq!(ambience_target(WeatherKind::Rain, 0.0), None);
        assert_eq!(ambience_target(WeatherKind::Rain, AMBIENCE_MIN_INTENSITY / 2.0), None);
        assert_eq!(
            ambience_target(WeatherKind::Rain, AMBIENCE_MIN_INTENSITY),
            Some(WeatherKind::Rain)
        );
    }

    #[test]
    fn volume_follows_intensity_and_stays_in_range() {
        assert_eq!(ambience_volume(0.0), 0.0);
        assert_eq!(ambience_volume(1.0), MAX_AMBIENCE_VOLUME);
        assert!(ambience_volume(0.3) < ambience_volume(0.6));
        assert_eq!(ambience_volume(-1.0), 0.0);
        assert_eq!(ambience_volume(5.0), MAX_AMBIENCE_VOLUME);
    }
}
