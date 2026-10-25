use macroquad::prelude::rand;

use crate::config::*;
use crate::entities::bird::Bird;
use crate::entities::cactus::CactusCluster;
use crate::entities::obstacle::Obstacle;

/// Khoảng cách tối thiểu giữa hai cụm, tỷ lệ thuận với tốc độ để luôn nhảy kịp
fn min_safe_gap(speed: f32) -> f32 {
    speed * 0.68
}

/// Khoảng cách tối đa giữa hai cụm
fn max_gap(speed: f32) -> f32 {
    (speed * 1.35).max(min_safe_gap(speed) + 120.0)
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ObstacleKind {
    Cactus,
    Bird,
}

/// Chọn loại chướng ngại: chim chỉ xuất hiện khi đủ điểm, với xác suất
/// `BIRD_SPAWN_CHANCE_PERCENT`%. `roll` là số ngẫu nhiên trong 0..100.
fn pick_kind(score: u32, roll: i32) -> ObstacleKind {
    if score >= BIRD_MIN_SCORE && roll < BIRD_SPAWN_CHANCE_PERCENT {
        ObstacleKind::Bird
    } else {
        ObstacleKind::Cactus
    }
}

/// Sinh chướng ngại vật, đảm bảo khoảng cách an toàn để người chơi nhảy qua được.
pub struct Spawner {
    next_gap: f32,
}

impl Spawner {
    pub fn new() -> Self {
        Self { next_gap: 350.0 }
    }

    pub fn reset(&mut self) {
        self.next_gap = 400.0;
    }

    /// Sinh thêm một chướng ngại vật nếu khoảng cách tới chướng ngại cuối đã đủ lớn.
    pub fn update(&mut self, obstacles: &mut Vec<Obstacle>, speed: f32, score: u32) {
        let should_spawn = match obstacles.last() {
            None => true,
            Some(last_obs) => {
                let current_distance = SCREEN_WIDTH - last_obs.right_edge();
                current_distance >= self.next_gap
            }
        };

        if should_spawn {
            let start_x = SCREEN_WIDTH + 20.0;
            let obstacle = match pick_kind(score, rand::gen_range(0, 100)) {
                ObstacleKind::Cactus => Obstacle::Cactus(CactusCluster::spawn(start_x)),
                ObstacleKind::Bird => Obstacle::Bird(Bird::spawn(start_x)),
            };
            obstacles.push(obstacle);
            // Khoảng cách tối thiểu tỷ lệ thuận với tốc độ hiện tại để luôn nhảy kịp
            self.next_gap = rand::gen_range(min_safe_gap(speed), max_gap(speed));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::dino::Dino;

    /// Mô phỏng một cú nhảy đầy đủ và trả về thời gian ở trên không (giây)
    fn simulated_air_time(dt: f32) -> f32 {
        simulated_air_time_with(dt, 1.0)
    }

    /// Như trên nhưng với lực nhảy bằng `multiplier` lần bình thường (gió làm cú nhảy yếu đi)
    fn simulated_air_time_with(dt: f32, multiplier: f32) -> f32 {
        let mut dino = Dino::new();
        dino.jump_with(multiplier);
        let mut elapsed = 0.0;
        let mut steps = 0;
        while !dino.is_grounded() {
            dino.update(dt, false);
            elapsed += dt;
            steps += 1;
            assert!(steps < 10_000, "khủng long không bao giờ chạm đất");
        }
        elapsed
    }

    fn speeds() -> impl Iterator<Item = f32> {
        (0..).map(|i| INITIAL_SPEED + i as f32 * 10.0).take_while(|s| *s <= MAX_SPEED)
    }

    #[test]
    fn min_gap_covers_one_full_jump_at_every_speed_and_frame_rate() {
        for dt in [1.0 / 144.0, 1.0 / 60.0, 1.0 / 30.0, 0.05] {
            let air_time = simulated_air_time(dt);
            for speed in speeds() {
                assert!(
                    min_safe_gap(speed) >= speed * air_time,
                    "dt={dt}: gap {} < quãng đường một cú nhảy {} ở tốc độ {speed}",
                    min_safe_gap(speed),
                    speed * air_time
                );
            }
        }
    }

    #[test]
    fn a_weaker_wind_jump_never_stays_in_the_air_longer() {
        // Thời tiết chỉ làm cú nhảy yếu đi, nên khoảng cách tối thiểu vẫn luôn đủ
        for dt in [1.0 / 144.0, 1.0 / 60.0, 1.0 / 30.0, 0.05] {
            let normal = simulated_air_time(dt);
            for multiplier in [0.97, 0.94, 0.9] {
                let weak = simulated_air_time_with(dt, multiplier);
                assert!(weak <= normal + 1e-6, "dt={dt}, x{multiplier}: {weak} > {normal}");
            }
        }
    }

    #[test]
    fn min_gap_covers_the_analytic_jump_time_too() {
        let analytic_air_time = 2.0 * -JUMP_FORCE / GRAVITY;
        for speed in speeds() {
            assert!(min_safe_gap(speed) >= speed * analytic_air_time);
        }
    }

    #[test]
    fn max_gap_leaves_at_least_120px_of_variety() {
        for speed in speeds() {
            assert!(max_gap(speed) - min_safe_gap(speed) >= 120.0 - 1e-3);
        }
    }

    #[test]
    fn spawns_the_first_obstacle_on_an_empty_field() {
        let mut spawner = Spawner::new();
        let mut obstacles = Vec::new();
        spawner.update(&mut obstacles, INITIAL_SPEED, 0);
        assert_eq!(obstacles.len(), 1);
        assert!(obstacles[0].right_edge() > SCREEN_WIDTH);
    }

    #[test]
    fn does_not_spawn_twice_in_a_row() {
        let mut spawner = Spawner::new();
        let mut obstacles = Vec::new();
        spawner.update(&mut obstacles, INITIAL_SPEED, 0);
        spawner.update(&mut obstacles, INITIAL_SPEED, 0);
        assert_eq!(obstacles.len(), 1);
    }

    #[test]
    fn next_gap_is_drawn_within_bounds() {
        for speed in [INITIAL_SPEED, 650.0, MAX_SPEED] {
            for _ in 0..50 {
                let mut spawner = Spawner::new();
                let mut obstacles = Vec::new();
                spawner.update(&mut obstacles, speed, 0);
                assert!(spawner.next_gap >= min_safe_gap(speed));
                assert!(spawner.next_gap <= max_gap(speed));
            }
        }
    }

    #[test]
    fn waits_until_distance_to_last_cluster_reaches_the_gap() {
        // Cụm cuối có mép phải ở 614 → còn cách mép màn hình 186px.
        let make = || vec![Obstacle::Cactus(CactusCluster::with_trees(600.0, &[(14.0, 30.0)]))];

        let mut spawner = Spawner::new();
        spawner.next_gap = 200.0;
        let mut obstacles = make();
        spawner.update(&mut obstacles, INITIAL_SPEED, 0);
        assert_eq!(obstacles.len(), 1, "chưa đủ khoảng cách, không được spawn");

        let mut spawner = Spawner::new();
        spawner.next_gap = 150.0;
        let mut obstacles = make();
        spawner.update(&mut obstacles, INITIAL_SPEED, 0);
        assert_eq!(obstacles.len(), 2, "đủ khoảng cách, phải spawn");
    }

    #[test]
    fn reset_restores_the_initial_gap() {
        let mut spawner = Spawner::new();
        let mut obstacles = Vec::new();
        spawner.update(&mut obstacles, MAX_SPEED, 0);
        spawner.reset();
        assert_eq!(spawner.next_gap, 400.0);
    }

    // ---------- Chọn loại chướng ngại ----------

    #[test]
    fn no_birds_before_the_minimum_score() {
        for roll in 0..100 {
            assert_eq!(pick_kind(BIRD_MIN_SCORE - 1, roll), ObstacleKind::Cactus);
        }
    }

    #[test]
    fn birds_appear_with_the_configured_chance_after_the_minimum_score() {
        let birds = (0..100)
            .filter(|roll| pick_kind(BIRD_MIN_SCORE, *roll) == ObstacleKind::Bird)
            .count();
        assert_eq!(birds as i32, BIRD_SPAWN_CHANCE_PERCENT);
        // Ranh giới của xác suất
        assert_eq!(pick_kind(BIRD_MIN_SCORE, BIRD_SPAWN_CHANCE_PERCENT - 1), ObstacleKind::Bird);
        assert_eq!(pick_kind(BIRD_MIN_SCORE, BIRD_SPAWN_CHANCE_PERCENT), ObstacleKind::Cactus);
    }

    #[test]
    fn update_below_the_minimum_score_only_spawns_cacti() {
        for _ in 0..200 {
            let mut spawner = Spawner::new();
            let mut obstacles = Vec::new();
            spawner.update(&mut obstacles, INITIAL_SPEED, BIRD_MIN_SCORE - 1);
            assert!(matches!(obstacles[0], Obstacle::Cactus(_)));
        }
    }

    #[test]
    fn update_above_the_minimum_score_spawns_both_kinds() {
        let (mut cacti, mut birds) = (0, 0);
        for _ in 0..400 {
            let mut spawner = Spawner::new();
            let mut obstacles = Vec::new();
            spawner.update(&mut obstacles, INITIAL_SPEED, BIRD_MIN_SCORE + 1000);
            match obstacles[0] {
                Obstacle::Cactus(_) => cacti += 1,
                Obstacle::Bird(_) => birds += 1,
            }
        }
        assert!(cacti > 0 && birds > 0, "cacti={cacti}, birds={birds}");
    }

    #[test]
    fn spawned_bird_starts_off_screen_on_the_right() {
        let mut found = false;
        for _ in 0..400 {
            let mut spawner = Spawner::new();
            let mut obstacles = Vec::new();
            spawner.update(&mut obstacles, INITIAL_SPEED, BIRD_MIN_SCORE);
            if let Obstacle::Bird(_) = obstacles[0] {
                assert!(obstacles[0].right_edge() > SCREEN_WIDTH);
                found = true;
            }
        }
        assert!(found, "không sinh ra con chim nào trong 400 lần thử");
    }
}
