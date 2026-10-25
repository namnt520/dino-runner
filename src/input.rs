use macroquad::prelude::*;

/// Ảnh chụp trạng thái phím của một khung hình.
/// Game chỉ làm việc với struct này, không gọi trực tiếp macroquad để đọc phím.
pub struct Input {
    /// Vừa nhấn phím nhảy trong khung hình này (Space hoặc ↑)
    pub jump_pressed: bool,
    /// Đang giữ phím nhảy (Space hoặc ↑)
    pub jump_held: bool,
    /// Đang giữ phím ↓ (cúi khi ở trên đất, rơi nhanh khi ở trên không)
    pub down: bool,
}

impl Input {
    pub fn poll() -> Self {
        Self {
            jump_pressed: is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up),
            jump_held: is_key_down(KeyCode::Space) || is_key_down(KeyCode::Up),
            down: is_key_down(KeyCode::Down),
        }
    }
}
