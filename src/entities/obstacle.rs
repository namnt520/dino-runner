use macroquad::prelude::Rect;

use crate::entities::bird::Bird;
use crate::entities::cactus::CactusCluster;
use crate::theme::Theme;

/// Mọi loại chướng ngại vật. Game, spawner và hệ thống va chạm chỉ làm việc với enum này,
/// nên thêm loại mới chỉ cần thêm một biến thể và các nhánh `match` bên dưới.
pub enum Obstacle {
    Cactus(CactusCluster),
    Bird(Bird),
}

impl Obstacle {
    pub fn update(&mut self, dt: f32, speed: f32) {
        match self {
            Obstacle::Cactus(cactus) => cactus.update(dt, speed),
            Obstacle::Bird(bird) => bird.update(dt, speed),
        }
    }

    /// Mép phải (dùng để tính khoảng cách spawn)
    pub fn right_edge(&self) -> f32 {
        match self {
            Obstacle::Cactus(cactus) => cactus.right_edge(),
            Obstacle::Bird(bird) => bird.right_edge(),
        }
    }

    /// Đã đi khỏi màn hình và có thể xóa
    pub fn is_off_screen(&self) -> bool {
        match self {
            Obstacle::Cactus(cactus) => cactus.is_off_screen(),
            Obstacle::Bird(bird) => bird.is_off_screen(),
        }
    }

    pub fn hitboxes(&self) -> Vec<Rect> {
        match self {
            Obstacle::Cactus(cactus) => cactus.hitboxes(),
            Obstacle::Bird(bird) => bird.hitboxes(),
        }
    }

    pub fn draw(&self, theme: &Theme) {
        match self {
            Obstacle::Cactus(cactus) => cactus.draw(theme),
            Obstacle::Bird(bird) => bird.draw(theme),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::bird::BirdLane;

    #[test]
    fn dispatches_to_cactus() {
        let obstacle = Obstacle::Cactus(CactusCluster::with_trees(100.0, &[(18.0, 50.0), (14.0, 30.0)]));
        assert_eq!(obstacle.hitboxes().len(), 2);
        assert!((obstacle.right_edge() - 138.0).abs() < 1e-3); // 100 + 18 + 6 + 14
    }

    #[test]
    fn dispatches_to_bird() {
        let obstacle = Obstacle::Bird(Bird::in_lane(100.0, BirdLane::Mid));
        assert_eq!(obstacle.hitboxes().len(), 1);
        assert!((obstacle.right_edge() - 144.0).abs() < 1e-3); // 100 + 44
    }

    #[test]
    fn update_moves_both_kinds() {
        let mut cactus = Obstacle::Cactus(CactusCluster::with_trees(500.0, &[(14.0, 30.0)]));
        let mut bird = Obstacle::Bird(Bird::in_lane(500.0, BirdLane::Low));
        let (cactus_before, bird_before) = (cactus.right_edge(), bird.right_edge());
        cactus.update(0.5, 200.0);
        bird.update(0.5, 200.0);
        assert!((cactus_before - cactus.right_edge() - 100.0).abs() < 1e-3);
        assert!((bird_before - bird.right_edge() - 100.0).abs() < 1e-3);
    }

    #[test]
    fn off_screen_follows_the_variant() {
        let gone_cactus = Obstacle::Cactus(CactusCluster::with_trees(-100.0, &[(14.0, 30.0)]));
        let gone_bird = Obstacle::Bird(Bird::in_lane(-100.0, BirdLane::High));
        let visible_bird = Obstacle::Bird(Bird::in_lane(100.0, BirdLane::High));
        assert!(gone_cactus.is_off_screen());
        assert!(gone_bird.is_off_screen());
        assert!(!visible_bird.is_off_screen());
    }
}
