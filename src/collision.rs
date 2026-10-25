use macroquad::prelude::Rect;

use crate::entities::dino::Dino;
use crate::entities::obstacle::Obstacle;

/// Kiểm tra va chạm AABB giữa khủng long và từng hitbox của mọi chướng ngại vật
pub fn dino_hits_any(dino: &Dino, obstacles: &[Obstacle]) -> bool {
    rect_hits_any(dino.hitbox(), obstacles)
}

/// Hộp `rect` có chạm hitbox của bất kỳ chướng ngại vật nào trong `obstacles` không
pub fn rect_hits_any(rect: Rect, obstacles: &[Obstacle]) -> bool {
    obstacles
        .iter()
        .any(|obs| obs.hitboxes().iter().any(|hitbox| rect.overlaps(hitbox)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::bird::{Bird, BirdLane};
    use crate::entities::cactus::CactusCluster;

    const DT: f32 = 1.0 / 60.0;

    // Cụm đặt tại x = 100: cây cao 18x50 và cây thấp 14x30 (cách nhau 6px).
    //   hitbox cây cao : x 103..115, y 274..320
    //   hitbox cây thấp: x 127..135, y 294..320
    fn mixed_cluster() -> Obstacle {
        Obstacle::Cactus(CactusCluster::with_trees(100.0, &[(18.0, 50.0), (14.0, 30.0)]))
    }

    // Chim đặt tại x = 80 → hitbox x 84..120, trùng theo phương ngang với khủng long (x 80).
    //   High: y 240..260   Mid: y 270..290   Low: y 294..314
    fn bird(lane: BirdLane) -> [Obstacle; 1] {
        [Obstacle::Bird(Bird::in_lane(80.0, lane))]
    }

    fn standing_dino() -> Dino {
        Dino::new() // hitbox x 88..116, y 280..316
    }

    fn ducking_dino() -> Dino {
        let mut dino = Dino::new();
        dino.update(DT, true); // hitbox x 86..130, y 299..316
        assert!(dino.is_ducking());
        dino
    }

    // ---------- Xương rồng ----------

    #[test]
    fn no_obstacles_means_no_collision() {
        assert!(!rect_hits_any(Rect::new(0.0, 0.0, 800.0, 400.0), &[]));
    }

    #[test]
    fn hitting_the_tall_tree_collides() {
        let obstacles = [mixed_cluster()];
        assert!(rect_hits_any(Rect::new(110.0, 280.0, 10.0, 20.0), &obstacles));
    }

    #[test]
    fn hitting_the_short_tree_collides() {
        let obstacles = [mixed_cluster()];
        assert!(rect_hits_any(Rect::new(125.0, 270.0, 12.0, 30.0), &obstacles));
    }

    #[test]
    fn flying_over_the_short_tree_next_to_a_tall_one_is_safe() {
        // Đáy hộp ở y = 290, cao hơn đỉnh hitbox cây thấp (294) và nằm ngoài cây cao.
        // Với hitbox cả cụm theo cây cao nhất (cách cũ) trường hợp này bị tính là va chạm.
        let obstacles = [mixed_cluster()];
        assert!(!rect_hits_any(Rect::new(125.0, 260.0, 12.0, 30.0), &obstacles));
    }

    #[test]
    fn passing_through_the_gap_between_trees_is_safe() {
        let obstacles = [mixed_cluster()];
        assert!(!rect_hits_any(Rect::new(116.0, 300.0, 10.0, 16.0), &obstacles));
    }

    #[test]
    fn jumping_above_the_tall_tree_is_safe() {
        let obstacles = [mixed_cluster()];
        assert!(!rect_hits_any(Rect::new(105.0, 200.0, 10.0, 60.0), &obstacles));
    }

    #[test]
    fn collision_with_a_later_obstacle_is_detected() {
        let obstacles = [
            Obstacle::Cactus(CactusCluster::with_trees(500.0, &[(14.0, 30.0)])),
            mixed_cluster(),
        ];
        assert!(rect_hits_any(Rect::new(110.0, 280.0, 10.0, 20.0), &obstacles));
    }

    #[test]
    fn dino_standing_on_the_ground_hits_a_cactus_in_front_of_it() {
        let dino = standing_dino();
        assert!(dino_hits_any(&dino, &[mixed_cluster()]));
        assert!(!dino_hits_any(
            &dino,
            &[Obstacle::Cactus(CactusCluster::with_trees(300.0, &[(18.0, 50.0)]))]
        ));
    }

    #[test]
    fn ducking_does_not_help_against_a_cactus() {
        assert!(dino_hits_any(&ducking_dino(), &[mixed_cluster()]));
    }

    // ---------- Chim bay: mỗi tầng một cách né ----------

    #[test]
    fn standing_dino_passes_under_a_high_bird() {
        assert!(!dino_hits_any(&standing_dino(), &bird(BirdLane::High)));
    }

    #[test]
    fn standing_dino_is_hit_by_a_mid_bird() {
        assert!(dino_hits_any(&standing_dino(), &bird(BirdLane::Mid)));
    }

    #[test]
    fn standing_dino_is_hit_by_a_low_bird() {
        assert!(dino_hits_any(&standing_dino(), &bird(BirdLane::Low)));
    }

    #[test]
    fn ducking_dino_passes_under_a_mid_bird() {
        assert!(!dino_hits_any(&ducking_dino(), &bird(BirdLane::Mid)));
    }

    #[test]
    fn ducking_dino_passes_under_a_high_bird() {
        assert!(!dino_hits_any(&ducking_dino(), &bird(BirdLane::High)));
    }

    #[test]
    fn ducking_dino_is_still_hit_by_a_low_bird() {
        assert!(dino_hits_any(&ducking_dino(), &bird(BirdLane::Low)));
    }

    #[test]
    fn jumping_dino_clears_mid_and_low_birds() {
        let mut dino = Dino::new();
        dino.jump();
        for _ in 0..18 {
            dino.update(DT, false); // khoảng 0.3 giây sau khi nhảy, gần đỉnh
        }
        assert!(!dino_hits_any(&dino, &bird(BirdLane::Mid)));
        assert!(!dino_hits_any(&dino, &bird(BirdLane::Low)));
    }
}
