use macroquad::prelude::*;

use crate::config::*;
use crate::game::GameState;
use crate::score::Score;
use crate::theme::Theme;

pub fn draw_centered_text(text: &str, y: f32, font_size: u16, color: Color) {
    let dims = measure_text(text, None, font_size, 1.0);
    let x = (SCREEN_WIDTH - dims.width) / 2.0;
    draw_text(text, x, y, font_size as f32, color);
}

/// Điểm số nhấp nháy (biến mất rồi hiện lại) trong lúc `flash_timer` còn dương,
/// giống Chrome mỗi khi qua mốc 100 điểm. Trả về `true` ở nhịp đang ẩn.
pub fn score_blinks_off(flash_timer: f32) -> bool {
    flash_timer > 0.0 && (flash_timer * 8.0) as u32 % 2 == 1
}

/// Hiển thị Điểm số & High Score ở góc phải trên.
/// `blink_off` = true thì ẩn phần điểm hiện tại (hiệu ứng nhấp nháy khi qua mốc 100).
pub fn draw_hud(score: &Score, theme: &Theme, blink_off: bool) {
    let full = format!("HI {:05}  {:05}", score.high(), score.current());
    // Vị trí căn theo chuỗi đầy đủ, để phần "HI" không bị xê dịch khi điểm nhấp nháy
    let x = SCREEN_WIDTH - measure_text(&full, None, 24, 1.0).width - 24.0;
    let shown = if blink_off {
        format!("HI {:05}", score.high())
    } else {
        full
    };
    draw_text(&shown, x, 36.0, 24.0, theme.primary);
}

/// Tên thời tiết hiện tại ở góc trên trái, mờ dần theo cường độ
pub fn draw_weather_label(label: Option<&str>, intensity: f32, theme: &Theme) {
    if let Some(text) = label {
        let c = theme.primary;
        let color = Color::new(c.r, c.g, c.b, intensity.clamp(0.0, 1.0));
        draw_text(text, 24.0, 36.0, 20.0, color);
    }
}

/// Hiển thị UI theo trạng thái State Machine
pub fn draw_state_overlay(state: GameState, score: &Score, theme: &Theme) {
    let primary_color = theme.primary;
    match state {
        GameState::Waiting => {
            draw_centered_text("CHROME DINO RUNNER", 140.0, 32, primary_color);
            draw_centered_text("Press Space to Start", 180.0, 24, primary_color);
            draw_centered_text("Space / Up: Jump     Down: Duck", 212.0, 20, primary_color);
        }
        GameState::GameOver => {
            draw_centered_text("G A M E  O V E R", 130.0, 34, primary_color);
            let final_score_msg = format!(
                "Score: {}   |   High Score: {}",
                score.current(),
                score.high()
            );
            draw_centered_text(&final_score_msg, 168.0, 22, primary_color);
            draw_centered_text("Press Space to Restart", 205.0, 24, primary_color);
        }
        GameState::Playing => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_is_visible_when_not_flashing() {
        assert!(!score_blinks_off(0.0));
        assert!(!score_blinks_off(-1.0));
    }

    #[test]
    fn score_blinks_on_and_off_while_flashing() {
        let mut hidden = 0;
        let mut shown = 0;
        let mut timer = 0.8;
        while timer > 0.0 {
            if score_blinks_off(timer) {
                hidden += 1;
            } else {
                shown += 1;
            }
            timer -= 0.01;
        }
        assert!(hidden > 0 && shown > 0, "hidden={hidden}, shown={shown}");
    }
}
