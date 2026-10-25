use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};

use crate::game::GameEvent;
use crate::synth;

/// Âm thanh của game. Âm thanh được tổng hợp bằng code (`synth.rs`) nên không cần file đi kèm.
/// Nếu tải âm thanh lỗi (ví dụ máy không có thiết bị âm thanh), game vẫn chạy bình thường, chỉ im lặng.
pub struct Sfx {
    jump: Option<Sound>,
    die: Option<Sound>,
    milestone: Option<Sound>,
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
}
