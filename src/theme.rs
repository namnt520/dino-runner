use macroquad::prelude::Color;

/// Bảng màu của game, gom một chỗ để dễ đổi giao diện.
pub struct Theme {
    pub bg: Color,
    pub primary: Color,
    pub cloud: Color,
}

impl Theme {
    /// Màu sắc phong cách Chrome Offline truyền thống (ban ngày)
    pub fn chrome() -> Self {
        Self {
            bg: Color::from_rgba(247, 247, 247, 255),
            primary: Color::from_rgba(83, 83, 83, 255),
            cloud: Color::from_rgba(215, 215, 215, 255),
        }
    }

    /// Màu ban đêm: nền tối, nét vẽ sáng
    pub fn night() -> Self {
        Self {
            bg: Color::from_rgba(32, 33, 36, 255),
            primary: Color::from_rgba(172, 172, 172, 255),
            cloud: Color::from_rgba(70, 72, 76, 255),
        }
    }

    /// Pha trộn giữa `self` (t = 0) và `other` (t = 1)
    pub fn lerp(&self, other: &Theme, t: f32) -> Theme {
        let t = t.clamp(0.0, 1.0);
        Theme {
            bg: lerp_color(self.bg, other.bg, t),
            primary: lerp_color(self.primary, other.primary, t),
            cloud: lerp_color(self.cloud, other.cloud, t),
        }
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same(a: Color, b: Color) -> bool {
        (a.r - b.r).abs() < 1e-5
            && (a.g - b.g).abs() < 1e-5
            && (a.b - b.b).abs() < 1e-5
            && (a.a - b.a).abs() < 1e-5
    }

    #[test]
    fn lerp_at_zero_is_the_first_theme() {
        let (day, night) = (Theme::chrome(), Theme::night());
        let mixed = day.lerp(&night, 0.0);
        assert!(same(mixed.bg, day.bg));
        assert!(same(mixed.primary, day.primary));
        assert!(same(mixed.cloud, day.cloud));
    }

    #[test]
    fn lerp_at_one_is_the_second_theme() {
        let (day, night) = (Theme::chrome(), Theme::night());
        let mixed = day.lerp(&night, 1.0);
        assert!(same(mixed.bg, night.bg));
        assert!(same(mixed.primary, night.primary));
        assert!(same(mixed.cloud, night.cloud));
    }

    #[test]
    fn lerp_halfway_is_the_midpoint() {
        let (day, night) = (Theme::chrome(), Theme::night());
        let mixed = day.lerp(&night, 0.5);
        assert!((mixed.bg.r - (day.bg.r + night.bg.r) / 2.0).abs() < 1e-5);
        assert!((mixed.primary.g - (day.primary.g + night.primary.g) / 2.0).abs() < 1e-5);
    }

    #[test]
    fn lerp_clamps_out_of_range_factors() {
        let (day, night) = (Theme::chrome(), Theme::night());
        assert!(same(day.lerp(&night, -3.0).bg, day.bg));
        assert!(same(day.lerp(&night, 7.0).bg, night.bg));
    }

    #[test]
    fn night_is_dark_background_with_light_strokes() {
        let night = Theme::night();
        assert!(night.bg.r < night.primary.r);
        let day = Theme::chrome();
        assert!(day.bg.r > day.primary.r);
    }
}
