use macroquad::prelude::*;

use crate::config::*;
use crate::theme::Theme;

// ==========================================
// CHƯỚNG NGẠI VẬT: CỤM XƯƠNG RỒNG (CACTUS)
// ==========================================
// Độ thu gọn hitbox so với hình vẽ của mỗi cây (pixel-forgiving)
const HITBOX_INSET_X: f32 = 3.0;
const HITBOX_INSET_TOP: f32 = 4.0;

struct CactusTree {
    offset_x: f32,
    width: f32,
    height: f32,
    left_arm_y: f32,
    right_arm_y: f32,
}

pub struct CactusCluster {
    x: f32,
    total_width: f32,
    trees: Vec<CactusTree>,
}

impl CactusCluster {
    pub fn spawn(start_x: f32) -> Self {
        let count = rand::gen_range(1, 4); // Sinh ngẫu nhiên cụm từ 1 đến 3 cây
        let mut trees = Vec::with_capacity(count);
        let mut current_offset = 0.0;

        for _ in 0..count {
            let is_tall = rand::gen_range(0, 10) > 4;
            let width = if is_tall { 18.0 } else { 14.0 };
            let height = if is_tall {
                rand::gen_range(44.0, 54.0)
            } else {
                rand::gen_range(30.0, 38.0)
            };

            trees.push(CactusTree {
                offset_x: current_offset,
                width,
                height,
                left_arm_y: rand::gen_range(0.3, 0.55),
                right_arm_y: rand::gen_range(0.25, 0.5),
            });

            // Khoảng cách giữa các cây trong cùng một cụm
            current_offset += width + 6.0;
        }

        let total_width = (current_offset - 6.0).max(14.0);

        Self {
            x: start_x,
            total_width,
            trees,
        }
    }

    pub fn update(&mut self, dt: f32, speed: f32) {
        self.x -= speed * dt;
    }

    /// Mép phải của cụm (dùng để tính khoảng cách spawn)
    pub fn right_edge(&self) -> f32 {
        self.x + self.total_width
    }

    /// Cụm đã đi khỏi màn hình và có thể xóa
    pub fn is_off_screen(&self) -> bool {
        !(self.right_edge() > -20.0)
    }

    /// Mỗi cây một hitbox AABB riêng (thu gọn nhẹ ở rìa), nên khoảng trống
    /// bên cạnh cây thấp trong cụm hỗn hợp không còn gây va chạm oan.
    pub fn hitboxes(&self) -> Vec<Rect> {
        self.trees
            .iter()
            .map(|tree| {
                Rect::new(
                    self.x + tree.offset_x + HITBOX_INSET_X,
                    GROUND_Y - tree.height + HITBOX_INSET_TOP,
                    tree.width - 2.0 * HITBOX_INSET_X,
                    tree.height - HITBOX_INSET_TOP,
                )
            })
            .collect()
    }

    /// Dựng cụm với kích thước (rộng, cao) cho trước, dùng cho unit test.
    #[cfg(test)]
    pub fn with_trees(x: f32, dims: &[(f32, f32)]) -> Self {
        let mut trees = Vec::new();
        let mut offset = 0.0;
        for &(width, height) in dims {
            trees.push(CactusTree {
                offset_x: offset,
                width,
                height,
                left_arm_y: 0.4,
                right_arm_y: 0.4,
            });
            offset += width + 6.0;
        }
        Self {
            x,
            total_width: (offset - 6.0).max(14.0),
            trees,
        }
    }

    pub fn draw(&self, theme: &Theme) {
        let color = theme.primary;
        for tree in &self.trees {
            let tx = self.x + tree.offset_x;
            let ty = GROUND_Y - tree.height;

            // Thân chính của cây xương rồng
            let trunk_w = (tree.width * 0.45).max(6.0);
            let trunk_x = tx + (tree.width - trunk_w) / 2.0;
            draw_rectangle(trunk_x, ty, trunk_w, tree.height, color);

            // Nhánh bên trái
            let l_y = ty + tree.height * tree.left_arm_y;
            draw_rectangle(tx, l_y, trunk_x - tx + 2.0, 4.0, color);
            // Đầu nhánh không được nhô lên trên đỉnh thân (cây thấp)
            let l_top = (l_y - 10.0).max(ty);
            draw_rectangle(tx, l_top, 4.0, l_y + 2.0 - l_top, color);

            // Nhánh bên phải
            let r_y = ty + tree.height * tree.right_arm_y;
            let r_x = trunk_x + trunk_w - 2.0;
            let r_w = (tx + tree.width) - r_x;
            draw_rectangle(r_x, r_y, r_w, 4.0, color);
            let r_top = (r_y - 12.0).max(ty);
            draw_rectangle(tx + tree.width - 4.0, r_top, 4.0, r_y + 2.0 - r_top, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn one_hitbox_per_tree() {
        let cluster = CactusCluster::with_trees(100.0, &[(18.0, 50.0), (14.0, 30.0), (14.0, 34.0)]);
        assert_eq!(cluster.hitboxes().len(), 3);
    }

    #[test]
    fn single_tree_hitbox_is_inset_from_drawn_shape() {
        let cluster = CactusCluster::with_trees(100.0, &[(18.0, 50.0)]);
        let b = cluster.hitboxes()[0];
        assert!(approx(b.x, 103.0));
        assert!(approx(b.w, 12.0));
        assert!(approx(b.y, GROUND_Y - 50.0 + 4.0));
        assert!(approx(b.y + b.h, GROUND_Y)); // chạm đúng mặt đất
    }

    #[test]
    fn short_tree_in_mixed_cluster_gets_lower_hitbox() {
        let cluster = CactusCluster::with_trees(100.0, &[(18.0, 50.0), (14.0, 30.0)]);
        let boxes = cluster.hitboxes();
        assert!(boxes[0].y < boxes[1].y, "cây cao phải có hitbox cao hơn cây thấp");
        for b in &boxes {
            assert!(approx(b.y + b.h, GROUND_Y));
        }
    }

    #[test]
    fn hitbox_stays_inside_the_drawn_tree() {
        let cluster = CactusCluster::with_trees(250.0, &[(18.0, 52.0), (14.0, 31.0), (18.0, 45.0)]);
        let boxes = cluster.hitboxes();
        for (tree, b) in cluster.trees.iter().zip(&boxes) {
            let tx = cluster.x + tree.offset_x;
            assert!(b.x >= tx && b.x + b.w <= tx + tree.width);
            assert!(b.y >= GROUND_Y - tree.height);
            assert!(b.w > 0.0 && b.h > 0.0);
        }
    }

    #[test]
    fn hitboxes_of_neighbouring_trees_do_not_overlap() {
        let cluster = CactusCluster::with_trees(0.0, &[(18.0, 50.0), (14.0, 30.0), (18.0, 46.0)]);
        let boxes = cluster.hitboxes();
        for i in 0..boxes.len() {
            for j in (i + 1)..boxes.len() {
                assert!(!boxes[i].overlaps(&boxes[j]), "cây {i} và {j} chồng nhau");
            }
        }
    }

    #[test]
    fn update_moves_cluster_left_by_speed_times_dt() {
        let mut cluster = CactusCluster::with_trees(500.0, &[(14.0, 30.0)]);
        cluster.update(0.5, 400.0);
        assert!(approx(cluster.x, 300.0));
    }

    #[test]
    fn off_screen_threshold_is_minus_20_on_right_edge() {
        // total_width = 14 → right_edge = x + 14
        let still_visible = CactusCluster::with_trees(-33.0, &[(14.0, 30.0)]); // -19
        let gone = CactusCluster::with_trees(-34.0, &[(14.0, 30.0)]); // -20
        assert!(!still_visible.is_off_screen());
        assert!(gone.is_off_screen());
    }

    #[test]
    fn spawn_builds_one_to_three_consistent_trees() {
        for _ in 0..200 {
            let cluster = CactusCluster::spawn(SCREEN_WIDTH + 20.0);
            assert!((1..=3).contains(&cluster.trees.len()));
            let last = cluster.trees.last().unwrap();
            assert!(approx(
                cluster.total_width,
                (last.offset_x + last.width).max(14.0)
            ));
            for tree in &cluster.trees {
                assert!(tree.height >= 30.0 && tree.height <= 54.0);
            }
            assert_eq!(cluster.hitboxes().len(), cluster.trees.len());
        }
    }
}
