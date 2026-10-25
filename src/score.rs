use std::fs;
use std::path::{Path, PathBuf};

/// File lưu high score, nằm trong thư mục hiện tại của tiến trình
/// (khi chạy `cargo run` ở thư mục dự án thì đó chính là thư mục dự án).
/// Định dạng: dòng 1 là high score, dòng 2 (có thể thiếu) là chỉ số skin đang chọn.
const HIGH_SCORE_FILE: &str = "highscore.txt";

/// Điểm hiện tại và điểm cao nhất (được lưu ra file, cùng với skin đang chọn).
pub struct Score {
    current: f32,
    high: u32,
    /// Chỉ số skin đang chọn (xem `entities::skin`)
    skin: usize,
    /// Mốc 100 điểm gần nhất đã được báo (xem `take_milestone`)
    last_milestone: u32,
    path: PathBuf,
}

impl Score {
    pub fn new() -> Self {
        Self::with_file(HIGH_SCORE_FILE)
    }

    /// Dùng file lưu tùy chọn (để unit test không đụng vào file thật).
    pub fn with_file(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let (high, skin) = load_saved(&path);
        Self {
            current: 0.0,
            high,
            skin,
            last_milestone: 0,
            path,
        }
    }

    /// Đặt lại điểm hiện tại (giữ nguyên high score)
    pub fn reset(&mut self) {
        self.current = 0.0;
        self.last_milestone = 0;
    }

    /// Tăng điểm theo thời gian sống và tốc độ
    pub fn advance(&mut self, dt: f32, speed: f32) {
        self.current += dt * (speed / 28.0);
    }

    pub fn current(&self) -> u32 {
        self.current as u32
    }

    /// Điểm chính xác (có phần thập phân), dùng cho các hiệu ứng chuyển mượt theo điểm
    pub fn exact(&self) -> f32 {
        self.current
    }

    /// Trả về `true` đúng một lần mỗi khi điểm vừa vượt qua một mốc 100 điểm mới
    pub fn take_milestone(&mut self) -> bool {
        let milestone = self.current() / 100;
        if milestone > self.last_milestone {
            self.last_milestone = milestone;
            true
        } else {
            false
        }
    }

    pub fn high(&self) -> u32 {
        self.high
    }

    /// Cập nhật high score nếu điểm hiện tại cao hơn, đồng thời ghi ra file.
    pub fn commit_high(&mut self) {
        let final_score = self.current();
        if final_score > self.high {
            self.high = final_score;
            save(&self.path, self.high, self.skin);
        }
    }

    pub fn skin_index(&self) -> usize {
        self.skin
    }

    /// Đổi skin đang chọn và lưu ngay (không ghi file nếu không có gì thay đổi).
    pub fn set_skin_index(&mut self, skin: usize) {
        if skin != self.skin {
            self.skin = skin;
            save(&self.path, self.high, self.skin);
        }
    }
}

/// Đọc (high score, chỉ số skin) từ file; thiếu file hoặc nội dung hỏng thì dùng 0.
/// File cũ chỉ có mỗi high score vẫn đọc được (skin = 0).
fn load_saved(path: &Path) -> (u32, usize) {
    let Ok(content) = fs::read_to_string(path) else {
        return (0, 0);
    };
    let mut lines = content.lines();
    let high = lines.next().and_then(|l| l.trim().parse().ok()).unwrap_or(0);
    let skin = lines.next().and_then(|l| l.trim().parse().ok()).unwrap_or(0);
    (high, skin)
}

/// Ghi high score và skin ra file. Lỗi ghi chỉ được báo ra stderr, game vẫn chạy tiếp.
fn save(path: &Path, high: u32, skin: usize) {
    if let Err(err) = fs::write(path, format!("{high}\n{skin}")) {
        eprintln!("Không lưu được dữ liệu vào {}: {err}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// Đường dẫn file tạm duy nhất cho mỗi test, và tự xóa khi test kết thúc.
    struct TempFile(PathBuf);

    impl TempFile {
        fn new(name: &str) -> Self {
            let path = env::temp_dir().join(format!(
                "dino_runner_test_{}_{name}.txt",
                std::process::id()
            ));
            let _ = fs::remove_file(&path);
            Self(path)
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn missing_file_starts_from_zero() {
        let file = TempFile::new("missing");
        assert_eq!(Score::with_file(&file.0).high(), 0);
    }

    #[test]
    fn reads_existing_high_score_ignoring_whitespace() {
        let file = TempFile::new("existing");
        fs::write(&file.0, "123\n").unwrap();
        assert_eq!(Score::with_file(&file.0).high(), 123);
    }

    #[test]
    fn corrupted_file_starts_from_zero() {
        let file = TempFile::new("corrupted");
        fs::write(&file.0, "không phải số").unwrap();
        assert_eq!(Score::with_file(&file.0).high(), 0);
    }

    #[test]
    fn advance_adds_dt_times_speed_over_28() {
        let file = TempFile::new("advance");
        let mut score = Score::with_file(&file.0);
        score.advance(1.0, 280.0); // 1 * 280 / 28 = 10
        assert_eq!(score.current(), 10);
    }

    #[test]
    fn new_record_is_saved_and_loaded_back() {
        let file = TempFile::new("record");
        let mut score = Score::with_file(&file.0);
        score.advance(1.0, 280.0);
        score.commit_high();
        assert_eq!(score.high(), 10);
        assert_eq!(fs::read_to_string(&file.0).unwrap(), "10\n0");

        assert_eq!(Score::with_file(&file.0).high(), 10);
    }

    #[test]
    fn lower_score_does_not_overwrite_the_record() {
        let file = TempFile::new("lower");
        fs::write(&file.0, "50").unwrap();
        let mut score = Score::with_file(&file.0);
        score.advance(1.0, 280.0); // 10 < 50
        score.commit_high();
        assert_eq!(score.high(), 50);
        assert_eq!(fs::read_to_string(&file.0).unwrap(), "50");
    }

    #[test]
    fn zero_score_does_not_create_a_file() {
        let file = TempFile::new("zero");
        let mut score = Score::with_file(&file.0);
        score.commit_high();
        assert!(!file.0.exists());
    }

    #[test]
    fn reset_keeps_the_high_score() {
        let file = TempFile::new("reset");
        let mut score = Score::with_file(&file.0);
        score.advance(1.0, 280.0);
        score.commit_high();
        score.reset();
        assert_eq!(score.current(), 0);
        assert_eq!(score.high(), 10);
    }

    #[test]
    fn unwritable_location_does_not_panic_and_keeps_record_in_memory() {
        let mut score = Score::with_file("/khong_ton_tai_thu_muc_nay/highscore.txt");
        score.advance(1.0, 280.0);
        score.commit_high();
        assert_eq!(score.high(), 10);
    }

    // ---------- Skin được lưu cùng high score ----------

    #[test]
    fn old_files_with_only_a_high_score_load_with_the_default_skin() {
        let file = TempFile::new("old_format");
        fs::write(&file.0, "123").unwrap();
        let score = Score::with_file(&file.0);
        assert_eq!((score.high(), score.skin_index()), (123, 0));
    }

    #[test]
    fn skin_choice_is_saved_and_loaded_back() {
        let file = TempFile::new("skin_saved");
        let mut score = Score::with_file(&file.0);
        score.set_skin_index(2);
        assert_eq!(fs::read_to_string(&file.0).unwrap(), "0\n2");
        assert_eq!(Score::with_file(&file.0).skin_index(), 2);
    }

    #[test]
    fn saving_a_record_keeps_the_chosen_skin_and_vice_versa() {
        let file = TempFile::new("skin_and_record");
        let mut score = Score::with_file(&file.0);
        score.set_skin_index(1);
        score.advance(1.0, 280.0);
        score.commit_high();

        let reloaded = Score::with_file(&file.0);
        assert_eq!((reloaded.high(), reloaded.skin_index()), (10, 1));

        let mut score = reloaded;
        score.set_skin_index(3);
        assert_eq!(Score::with_file(&file.0).high(), 10, "đổi skin không làm mất high score");
    }

    #[test]
    fn choosing_the_same_skin_does_not_touch_the_file() {
        let file = TempFile::new("skin_same");
        let mut score = Score::with_file(&file.0);
        score.set_skin_index(0);
        assert!(!file.0.exists());
    }

    #[test]
    fn a_corrupted_skin_line_falls_back_to_the_default_skin() {
        let file = TempFile::new("skin_corrupted");
        fs::write(&file.0, "80\nabc").unwrap();
        let score = Score::with_file(&file.0);
        assert_eq!((score.high(), score.skin_index()), (80, 0));
    }

    #[test]
    fn reset_keeps_the_chosen_skin() {
        let file = TempFile::new("skin_reset");
        let mut score = Score::with_file(&file.0);
        score.set_skin_index(2);
        score.reset();
        assert_eq!(score.skin_index(), 2);
    }

    #[test]
    fn exact_keeps_the_fractional_part() {
        let file = TempFile::new("exact");
        let mut score = Score::with_file(&file.0);
        score.advance(1.0, 14.0); // 0.5 điểm
        assert_eq!(score.current(), 0);
        assert!((score.exact() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn milestone_fires_once_per_hundred_points() {
        let file = TempFile::new("milestone");
        let mut score = Score::with_file(&file.0);

        score.advance(1.0, 2772.0); // 99 điểm
        assert!(!score.take_milestone());

        score.advance(1.0, 28.0); // 100 điểm
        assert!(score.take_milestone());
        assert!(!score.take_milestone(), "mỗi mốc chỉ báo một lần");

        score.advance(1.0, 2800.0); // 200 điểm
        assert!(score.take_milestone());
    }

    #[test]
    fn milestone_counting_restarts_after_reset() {
        let file = TempFile::new("milestone_reset");
        let mut score = Score::with_file(&file.0);
        score.advance(1.0, 2800.0); // 100 điểm
        assert!(score.take_milestone());

        score.reset();
        assert!(!score.take_milestone());
        score.advance(1.0, 2800.0);
        assert!(score.take_milestone(), "sau reset phải báo lại mốc 100");
    }
}
