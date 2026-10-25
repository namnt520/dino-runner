use macroquad::prelude::*;

// ==========================================
// SKIN CỦA KHỦNG LONG: PHỤ KIỆN MỞ KHÓA THEO HIGH SCORE
// ==========================================

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Skin {
    Classic,
    Shades,
    TopHat,
    Crown,
}

/// Thứ tự trong danh sách cũng là thứ tự mở khóa (điểm yêu cầu tăng dần)
pub const ALL: [Skin; 4] = [Skin::Classic, Skin::Shades, Skin::TopHat, Skin::Crown];

/// Vị trí đầu của khủng long, để vẽ phụ kiện đúng chỗ ở cả tư thế đứng lẫn cúi
#[derive(Clone, Copy)]
pub struct Head {
    /// Góc trên-trái của đầu
    pub x: f32,
    pub y: f32,
    /// Góc trên-trái của con mắt
    pub eye_x: f32,
    pub eye_y: f32,
}

const LENS: Color = Color { r: 0.08, g: 0.08, b: 0.10, a: 1.0 };
const HAT_RED: Color = Color { r: 0.78, g: 0.18, b: 0.22, a: 1.0 };
const HAT_BAND: Color = Color { r: 0.96, g: 0.84, b: 0.35, a: 1.0 };
const GOLD: Color = Color { r: 0.96, g: 0.76, b: 0.12, a: 1.0 };

impl Skin {
    /// Vị trí trong `ALL`, dùng để lưu vào file
    pub fn index(self) -> usize {
        match self {
            Skin::Classic => 0,
            Skin::Shades => 1,
            Skin::TopHat => 2,
            Skin::Crown => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Skin::Classic => "Classic",
            Skin::Shades => "Shades",
            Skin::TopHat => "Top Hat",
            Skin::Crown => "Crown",
        }
    }

    /// High score cần đạt để mở khóa
    pub fn unlock_score(self) -> u32 {
        match self {
            Skin::Classic => 0,
            Skin::Shades => 300,
            Skin::TopHat => 700,
            Skin::Crown => 1500,
        }
    }

    pub fn is_unlocked(self, high: u32) -> bool {
        high >= self.unlock_score()
    }

    /// Khôi phục skin từ chỉ số đã lưu. Chỉ số hỏng hoặc skin chưa mở khóa (ví dụ file bị sửa tay) thì về Classic.
    pub fn restore(index: usize, high: u32) -> Skin {
        match ALL.get(index) {
            Some(skin) if skin.is_unlocked(high) => *skin,
            _ => Skin::Classic,
        }
    }

    /// Skin đã mở khóa kế tiếp (vòng tròn). Nếu không còn skin nào khác thì giữ nguyên.
    pub fn next_unlocked(self, high: u32) -> Skin {
        self.step(high, 1)
    }

    pub fn prev_unlocked(self, high: u32) -> Skin {
        self.step(high, -1)
    }

    fn step(self, high: u32, direction: isize) -> Skin {
        let count = ALL.len() as isize;
        for offset in 1..count {
            let index = (self.index() as isize + direction * offset).rem_euclid(count);
            let candidate = ALL[index as usize];
            if candidate.is_unlocked(high) {
                return candidate;
            }
        }
        self
    }

    /// Skin kế tiếp còn bị khóa (để gợi ý mục tiêu), `None` khi đã mở hết
    pub fn next_locked(high: u32) -> Option<Skin> {
        ALL.iter().copied().find(|skin| !skin.is_unlocked(high))
    }

    /// Skin cao nhất vừa được mở khóa khi high score tăng từ `old_high` lên `new_high`
    pub fn newly_unlocked(old_high: u32, new_high: u32) -> Option<Skin> {
        ALL.iter()
            .copied()
            .filter(|skin| !skin.is_unlocked(old_high) && skin.is_unlocked(new_high))
            .last()
    }

    /// Vẽ phụ kiện lên đầu. Kính râm tự rơi mất khi khủng long chết (để lộ mắt "X").
    pub fn draw_accessory(self, head: Head, is_dead: bool) {
        match self {
            Skin::Classic => {}
            Skin::Shades => {
                if !is_dead {
                    // Gọng kéo về phía sau đầu + tròng kính che mắt
                    draw_rectangle(head.x, head.eye_y, head.eye_x - head.x - 3.0, 2.0, LENS);
                    draw_rectangle(head.eye_x - 3.0, head.eye_y - 1.0, 14.0, 5.0, LENS);
                }
            }
            Skin::TopHat => {
                draw_rectangle(head.x + 1.0, head.y - 3.0, 22.0, 3.0, HAT_RED); // vành
                draw_rectangle(head.x + 5.0, head.y - 13.0, 14.0, 10.0, HAT_RED); // thân mũ
                draw_rectangle(head.x + 5.0, head.y - 6.0, 14.0, 3.0, HAT_BAND); // dải mũ
            }
            Skin::Crown => {
                draw_rectangle(head.x + 3.0, head.y - 4.0, 18.0, 4.0, GOLD); // vành
                draw_rectangle(head.x + 3.0, head.y - 8.0, 4.0, 4.0, GOLD); // ba chóp
                draw_rectangle(head.x + 10.0, head.y - 9.0, 4.0, 5.0, GOLD);
                draw_rectangle(head.x + 17.0, head.y - 8.0, 4.0, 4.0, GOLD);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_is_always_available_and_unlock_scores_increase() {
        assert_eq!(Skin::Classic.unlock_score(), 0);
        assert!(Skin::Classic.is_unlocked(0));
        for pair in ALL.windows(2) {
            assert!(pair[0].unlock_score() < pair[1].unlock_score());
        }
    }

    #[test]
    fn indexes_match_the_position_in_all() {
        for (i, skin) in ALL.iter().enumerate() {
            assert_eq!(skin.index(), i);
        }
    }

    #[test]
    fn names_are_unique() {
        for (i, a) in ALL.iter().enumerate() {
            for b in &ALL[i + 1..] {
                assert_ne!(a.name(), b.name());
            }
        }
    }

    #[test]
    fn a_skin_unlocks_exactly_at_its_score() {
        let score = Skin::Shades.unlock_score();
        assert!(!Skin::Shades.is_unlocked(score - 1));
        assert!(Skin::Shades.is_unlocked(score));
    }

    #[test]
    fn restore_returns_the_saved_skin_when_unlocked() {
        assert_eq!(Skin::restore(2, 5000), Skin::TopHat);
        assert_eq!(Skin::restore(0, 0), Skin::Classic);
    }

    #[test]
    fn restore_falls_back_to_classic_for_locked_or_invalid_indexes() {
        assert_eq!(Skin::restore(3, 100), Skin::Classic, "chưa mở khóa");
        assert_eq!(Skin::restore(99, 100_000), Skin::Classic, "chỉ số hỏng");
    }

    #[test]
    fn with_nothing_unlocked_there_is_nowhere_to_switch() {
        assert_eq!(Skin::Classic.next_unlocked(0), Skin::Classic);
        assert_eq!(Skin::Classic.prev_unlocked(0), Skin::Classic);
    }

    #[test]
    fn cycling_skips_locked_skins_and_wraps_around() {
        let high = Skin::TopHat.unlock_score(); // Classic, Shades, TopHat mở; Crown khóa
        assert_eq!(Skin::Classic.next_unlocked(high), Skin::Shades);
        assert_eq!(Skin::Shades.next_unlocked(high), Skin::TopHat);
        assert_eq!(Skin::TopHat.next_unlocked(high), Skin::Classic, "bỏ qua Crown bị khóa, vòng về đầu");
        assert_eq!(Skin::Classic.prev_unlocked(high), Skin::TopHat, "lùi từ đầu thì vòng về cuối");
        assert_eq!(Skin::TopHat.prev_unlocked(high), Skin::Shades);
    }

    #[test]
    fn cycling_skips_a_gap_in_the_middle() {
        // Chỉ khi đạt Crown thì mọi skin đều mở; nhưng nếu từ Classic mà chỉ Shades mở thì đi qua đúng một bước
        let high = Skin::Shades.unlock_score();
        assert_eq!(Skin::Shades.next_unlocked(high), Skin::Classic);
        assert_eq!(Skin::Shades.prev_unlocked(high), Skin::Classic);
    }

    #[test]
    fn full_cycle_visits_every_unlocked_skin_once() {
        let high = u32::MAX;
        let mut skin = Skin::Classic;
        let mut seen = Vec::new();
        for _ in 0..ALL.len() {
            seen.push(skin);
            skin = skin.next_unlocked(high);
        }
        assert_eq!(skin, Skin::Classic);
        assert_eq!(seen, ALL.to_vec());
    }

    #[test]
    fn next_locked_points_at_the_next_goal() {
        assert_eq!(Skin::next_locked(0), Some(Skin::Shades));
        assert_eq!(Skin::next_locked(Skin::Shades.unlock_score()), Some(Skin::TopHat));
        assert_eq!(Skin::next_locked(Skin::Crown.unlock_score()), None);
    }

    #[test]
    fn newly_unlocked_reports_only_crossings() {
        assert_eq!(Skin::newly_unlocked(0, 299), None);
        assert_eq!(Skin::newly_unlocked(0, 300), Some(Skin::Shades));
        assert_eq!(Skin::newly_unlocked(300, 650), None, "không có skin mới");
        assert_eq!(Skin::newly_unlocked(300, 700), Some(Skin::TopHat));
    }

    #[test]
    fn newly_unlocked_reports_the_best_when_several_open_at_once() {
        assert_eq!(Skin::newly_unlocked(0, 800), Some(Skin::TopHat));
        assert_eq!(Skin::newly_unlocked(0, 100_000), Some(Skin::Crown));
    }

    #[test]
    fn a_lower_or_equal_score_unlocks_nothing() {
        assert_eq!(Skin::newly_unlocked(500, 500), None);
        assert_eq!(Skin::newly_unlocked(500, 100), None);
    }
}
