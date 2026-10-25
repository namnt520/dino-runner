// ==========================================
// CẤU HÌNH HẰNG SỐ VẬT LÝ & MÀN HÌNH
// ==========================================
pub const SCREEN_WIDTH: f32 = 800.0;
pub const SCREEN_HEIGHT: f32 = 400.0;
pub const GROUND_Y: f32 = 320.0;

pub const GRAVITY: f32 = 2100.0;
pub const JUMP_FORCE: f32 = -680.0;
pub const FAST_FALL_MULTIPLIER: f32 = 2.5;

pub const INITIAL_SPEED: f32 = 360.0;
pub const MAX_SPEED: f32 = 950.0;
pub const SPEED_ACCELERATION: f32 = 12.0; // Tốc độ tăng thêm mỗi giây

// ==========================================
// CHIM BAY
// ==========================================
/// Điểm tối thiểu để chim bắt đầu xuất hiện (người chơi làm quen với xương rồng trước)
pub const BIRD_MIN_SCORE: u32 = 300;
/// Tỷ lệ phần trăm mỗi lần spawn ra chim thay vì xương rồng (sau khi đủ điểm)
pub const BIRD_SPAWN_CHANCE_PERCENT: i32 = 25;

// ==========================================
// THỜI TIẾT
// ==========================================
/// Điểm tối thiểu để thời tiết (mưa, gió, bão cát) bắt đầu xuất hiện
pub const WEATHER_MIN_SCORE: u32 = 150;
