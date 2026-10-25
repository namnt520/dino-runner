//! Tổng hợp âm thanh bằng code, nên game không cần file âm thanh đi kèm.
//! Mọi hàm ở đây là hàm thuần (không đụng tới thiết bị âm thanh) nên test được.

use std::f32::consts::PI;

pub const SAMPLE_RATE: u32 = 44_100;

#[derive(Clone, Copy)]
#[allow(dead_code)] // `Sine` hiện chỉ dùng trong test, giữ lại để thiết kế âm thanh mới
pub enum Wave {
    Sine,
    Square,
    Saw,
}

/// `phase` nằm trong khoảng 0..1
fn wave_value(wave: Wave, phase: f32) -> f32 {
    match wave {
        Wave::Sine => (phase * 2.0 * PI).sin(),
        Wave::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Wave::Saw => 2.0 * phase - 1.0,
    }
}

/// Một âm có tần số trượt tuyến tính từ `f0` đến `f1` (Hz), dài `duration_ms`.
/// Biên độ tăng nhanh trong 5ms đầu rồi giảm dần về 0 để không bị tiếng "tách".
pub fn tone(wave: Wave, f0: f32, f1: f32, duration_ms: u32, volume: f32) -> Vec<f32> {
    let total = (SAMPLE_RATE as u64 * duration_ms as u64 / 1000) as usize;
    let attack_samples = SAMPLE_RATE as f32 * 0.005;
    let mut phase = 0.0_f32;
    let mut samples = Vec::with_capacity(total);

    for i in 0..total {
        let progress = i as f32 / total as f32; // 0..1
        let freq = f0 + (f1 - f0) * progress;
        phase = (phase + freq / SAMPLE_RATE as f32).fract();

        let attack = (i as f32 / attack_samples).min(1.0);
        let envelope = attack * (1.0 - progress);
        samples.push(wave_value(wave, phase) * volume * envelope);
    }
    samples
}

pub fn silence(duration_ms: u32) -> Vec<f32> {
    vec![0.0; (SAMPLE_RATE as u64 * duration_ms as u64 / 1000) as usize]
}

pub fn concat(parts: &[Vec<f32>]) -> Vec<f32> {
    parts.iter().flatten().copied().collect()
}

/// Đóng gói mẫu âm (-1..1) thành file WAV 16-bit PCM, mono, trong bộ nhớ.
pub fn to_wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut wav = Vec::with_capacity(44 + data_len as usize);

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes()); // kích thước khối fmt
    wav.extend_from_slice(&1_u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1_u16.to_le_bytes()); // mono
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte/giây
    wav.extend_from_slice(&2_u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16_u16.to_le_bytes()); // bit/mẫu

    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        wav.extend_from_slice(&value.to_le_bytes());
    }
    wav
}

/// Tiếng nhảy: âm vuông trượt lên
pub fn jump_sound() -> Vec<u8> {
    to_wav(&tone(Wave::Square, 380.0, 760.0, 120, 0.25))
}

/// Tiếng chết: âm răng cưa trượt xuống trầm
pub fn die_sound() -> Vec<u8> {
    to_wav(&tone(Wave::Saw, 320.0, 50.0, 380, 0.3))
}

/// Tiếng "ting-ting" mỗi 100 điểm
pub fn milestone_sound() -> Vec<u8> {
    to_wav(&concat(&[
        tone(Wave::Square, 880.0, 880.0, 70, 0.22),
        silence(30),
        tone(Wave::Square, 1320.0, 1320.0, 120, 0.22),
    ]))
}

// ==========================================
// ÂM NỀN: MƯA, GIÓ, BÃO CÁT (nhiễu trắng đã lọc, lặp liền mạch)
// ==========================================

/// Bộ sinh nhiễu trắng xorshift32. Cùng seed thì luôn ra cùng một âm thanh,
/// và không phụ thuộc vào `macroquad::rand` nên test được ổn định.
struct Noise(u32);

impl Noise {
    fn new(seed: u32) -> Self {
        Self(seed.max(1)) // xorshift bị kẹt ở 0 nếu seed = 0
    }

    /// Số ngẫu nhiên trong -1..1
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x as f32 / u32::MAX as f32 * 2.0 - 1.0
    }
}

fn white_noise(seed: u32, len: usize) -> Vec<f32> {
    let mut noise = Noise::new(seed);
    (0..len).map(|_| noise.next()).collect()
}

/// Bộ lọc thông thấp một cực: cắt bớt âm cao, còn lại tiếng trầm "ù ù"
fn low_pass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let dt = 1.0 / SAMPLE_RATE as f32;
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    let alpha = dt / (rc + dt);
    let mut y = 0.0;
    samples
        .iter()
        .map(|x| {
            y += alpha * (*x - y);
            y
        })
        .collect()
}

/// Bộ lọc thông cao: bỏ âm trầm, còn lại tiếng "xì xì"
fn high_pass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let low = low_pass(samples, cutoff_hz);
    samples.iter().zip(low).map(|(x, l)| *x - l).collect()
}

/// Số mẫu đầu bỏ đi vì bộ lọc chưa ổn định
const WARMUP: usize = 2_000;
/// Đoạn đuôi được hòa vào đầu (100ms) để vòng lặp không bị tiếng "tách"
const LOOP_FADE: usize = 4_410;

/// Nhiễu trắng đã qua `filter`, dài đúng `len` mẫu và nối được đầu với đuôi một cách liền mạch:
/// `LOOP_FADE` mẫu đầu là sự hòa trộn (equal-power) giữa đầu và đoạn dư ra ở đuôi,
/// nên mẫu cuối nối thẳng vào mẫu đầu như một đoạn liên tục.
fn looped_noise(seed: u32, len: usize, filter: impl Fn(&[f32]) -> Vec<f32>) -> Vec<f32> {
    let filtered = filter(&white_noise(seed, WARMUP + len + LOOP_FADE));
    let body = &filtered[WARMUP..];
    (0..len)
        .map(|i| {
            if i < LOOP_FADE {
                let t = i as f32 / LOOP_FADE as f32 * PI / 2.0;
                body[i] * t.sin() + body[len + i] * t.cos()
            } else {
                body[i]
            }
        })
        .collect()
}

/// Co giãn để mẫu lớn nhất có biên độ đúng `target_peak`
fn normalize(samples: Vec<f32>, target_peak: f32) -> Vec<f32> {
    let peak = samples.iter().fold(0.0_f32, |max, s| max.max(s.abs()));
    if peak <= 0.0 {
        return samples;
    }
    samples.into_iter().map(|s| s * target_peak / peak).collect()
}

/// Cường độ từng đợt gió giật (khoảng 0.2..1.0), tuần hoàn theo độ dài vòng lặp
fn gust(i: usize, len: usize) -> f32 {
    let t = i as f32 / len as f32 * 2.0 * PI;
    0.6 + 0.25 * t.sin() + 0.15 * (3.0 * t + 1.3).sin()
}

const RAIN_SECONDS: usize = 2;
const WIND_SECONDS: usize = 4;

/// Mưa: nhiễu "xì xì" đều, không có đợt mạnh yếu
fn rain_samples() -> Vec<f32> {
    let hiss = looped_noise(7, SAMPLE_RATE as usize * RAIN_SECONDS, |raw| {
        low_pass(&high_pass(raw, 700.0), 6000.0)
    });
    normalize(hiss, 0.30)
}

/// Gió: tiếng trầm "ù ù" lên xuống theo từng đợt giật
fn wind_samples() -> Vec<f32> {
    let len = SAMPLE_RATE as usize * WIND_SECONDS;
    let rumble = normalize(looped_noise(11, len, |raw| low_pass(raw, 350.0)), 1.0);
    let gusting = rumble.iter().enumerate().map(|(i, s)| s * gust(i, len)).collect();
    normalize(gusting, 0.50)
}

/// Bão cát: gió dày hơn (trầm lẫn trung) cộng thêm tiếng hạt cát lép bép
fn sandstorm_samples() -> Vec<f32> {
    let len = SAMPLE_RATE as usize * WIND_SECONDS;
    let body = normalize(looped_noise(23, len, |raw| low_pass(raw, 900.0)), 1.0);
    let grit = normalize(looped_noise(31, len, |raw| high_pass(raw, 2500.0)), 1.0);
    let mixed = (0..len)
        .map(|i| (body[i] * 0.8 + grit[i] * 0.25) * gust(i, len))
        .collect();
    normalize(mixed, 0.45)
}

pub fn rain_sound() -> Vec<u8> {
    to_wav(&rain_samples())
}

pub fn wind_sound() -> Vec<u8> {
    to_wav(&wind_samples())
}

pub fn sandstorm_sound() -> Vec<u8> {
    to_wav(&sandstorm_samples())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u16_at(bytes: &[u8], at: usize) -> u16 {
        u16::from_le_bytes([bytes[at], bytes[at + 1]])
    }

    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    }

    fn peak(samples: &[f32]) -> f32 {
        samples.iter().fold(0.0_f32, |max, s| max.max(s.abs()))
    }

    #[test]
    fn tone_has_the_requested_length() {
        assert_eq!(tone(Wave::Sine, 440.0, 440.0, 1000, 0.5).len(), 44_100);
        assert_eq!(tone(Wave::Square, 380.0, 760.0, 120, 0.25).len(), 5_292);
    }

    #[test]
    fn tone_never_exceeds_the_requested_volume() {
        for wave in [Wave::Sine, Wave::Square, Wave::Saw] {
            let samples = tone(wave, 300.0, 900.0, 200, 0.3);
            assert!(peak(&samples) <= 0.3 + 1e-4);
        }
    }

    #[test]
    fn tone_is_audible() {
        let samples = tone(Wave::Square, 380.0, 760.0, 120, 0.25);
        assert!(peak(&samples) > 0.1);
    }

    #[test]
    fn tone_fades_in_and_out_to_avoid_clicks() {
        let samples = tone(Wave::Square, 440.0, 440.0, 200, 0.3);
        assert_eq!(samples[0], 0.0);
        assert!(samples.last().unwrap().abs() < 0.01);
    }

    #[test]
    fn sine_tone_has_the_requested_frequency() {
        // 441 Hz trong 1 giây ⇒ khoảng 882 lần đổi dấu
        let samples = tone(Wave::Sine, 441.0, 441.0, 1000, 0.5);
        let crossings = samples
            .windows(2)
            .filter(|pair| pair[0] * pair[1] < 0.0)
            .count();
        assert!((879..=885).contains(&crossings), "đếm được {crossings}");
    }

    #[test]
    fn sweeping_up_crosses_zero_more_often_toward_the_end() {
        let samples = tone(Wave::Sine, 200.0, 1200.0, 1000, 0.5);
        let half = samples.len() / 2;
        let count = |part: &[f32]| part.windows(2).filter(|p| p[0] * p[1] < 0.0).count();
        assert!(count(&samples[half..]) > count(&samples[..half]));
    }

    #[test]
    fn silence_is_all_zeros_with_the_right_length() {
        let quiet = silence(30);
        assert_eq!(quiet.len(), 1_323);
        assert!(quiet.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn concat_joins_parts_in_order() {
        let joined = concat(&[vec![1.0, 2.0], vec![], vec![3.0]]);
        assert_eq!(joined, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn wav_header_is_valid_pcm16_mono() {
        let wav = to_wav(&[0.0, 0.5, -0.5, 1.0]);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(u32_at(&wav, 16), 16);
        assert_eq!(u16_at(&wav, 20), 1); // PCM
        assert_eq!(u16_at(&wav, 22), 1); // mono
        assert_eq!(u32_at(&wav, 24), SAMPLE_RATE);
        assert_eq!(u32_at(&wav, 28), SAMPLE_RATE * 2);
        assert_eq!(u16_at(&wav, 32), 2);
        assert_eq!(u16_at(&wav, 34), 16);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(u32_at(&wav, 40), 8); // 4 mẫu x 2 byte
    }

    #[test]
    fn wav_sizes_are_consistent() {
        let wav = to_wav(&vec![0.1; 1000]);
        assert_eq!(wav.len(), 44 + 2000);
        assert_eq!(u32_at(&wav, 4) as usize, wav.len() - 8); // kích thước RIFF
        assert_eq!(u32_at(&wav, 40), 2000);
    }

    #[test]
    fn wav_samples_are_scaled_and_clamped_to_i16() {
        let wav = to_wav(&[0.0, 1.0, -1.0, 5.0, -5.0]);
        let sample = |i: usize| i16::from_le_bytes([wav[44 + 2 * i], wav[45 + 2 * i]]);
        assert_eq!(sample(0), 0);
        assert_eq!(sample(1), i16::MAX);
        assert_eq!(sample(2), -i16::MAX);
        assert_eq!(sample(3), i16::MAX); // bị chặn ở 1.0
        assert_eq!(sample(4), -i16::MAX);
    }

    #[test]
    fn game_sounds_are_valid_wav_files_with_sensible_lengths() {
        // (file, thời lượng mong đợi theo ms)
        let sounds = [
            (jump_sound(), 120),
            (die_sound(), 380),
            (milestone_sound(), 70 + 30 + 120),
        ];
        for (wav, ms) in sounds {
            assert_eq!(&wav[0..4], b"RIFF");
            let samples = (u32_at(&wav, 40) / 2) as u64;
            let expected = SAMPLE_RATE as u64 * ms / 1000;
            assert!(
                samples.abs_diff(expected) <= 3,
                "{samples} mẫu, mong đợi khoảng {expected}"
            );
        }
    }

    // ---------- Âm nền (mưa, gió, bão cát) ----------

    fn mean(samples: &[f32]) -> f32 {
        samples.iter().sum::<f32>() / samples.len() as f32
    }

    /// Độ "gai" của âm: biến thiên giữa hai mẫu liên tiếp so với biên độ trung bình
    /// (nhiễu trắng rất gai, âm trầm thì mượt)
    fn roughness(samples: &[f32]) -> f32 {
        let step = samples.windows(2).map(|p| (p[1] - p[0]).abs()).sum::<f32>()
            / (samples.len() - 1) as f32;
        step / (samples.iter().map(|s| s.abs()).sum::<f32>() / samples.len() as f32)
    }

    fn max_step(samples: &[f32]) -> f32 {
        samples.windows(2).fold(0.0_f32, |max, p| max.max((p[1] - p[0]).abs()))
    }

    /// Tỷ lệ giữa quãng nửa giây to nhất và nhỏ nhất (1.0 = hoàn toàn đều)
    fn loudness_swing(samples: &[f32]) -> f32 {
        let window = SAMPLE_RATE as usize / 2;
        let levels: Vec<f32> = samples
            .chunks_exact(window)
            .map(|chunk| (chunk.iter().map(|s| s * s).sum::<f32>() / window as f32).sqrt())
            .collect();
        let max = levels.iter().cloned().fold(0.0_f32, f32::max);
        let min = levels.iter().cloned().fold(f32::MAX, f32::min);
        max / min
    }

    fn ambience() -> [(&'static str, Vec<f32>, f32); 3] {
        [
            ("rain", rain_samples(), 0.30),
            ("wind", wind_samples(), 0.50),
            ("sandstorm", sandstorm_samples(), 0.45),
        ]
    }

    #[test]
    fn noise_is_deterministic_per_seed_and_stays_in_range() {
        assert_eq!(white_noise(5, 1000), white_noise(5, 1000));
        assert_ne!(white_noise(5, 1000), white_noise(6, 1000));
        assert!(white_noise(5, 20_000).iter().all(|s| (-1.0..=1.0).contains(s)));
    }

    #[test]
    fn noise_is_centered_and_does_not_get_stuck_on_seed_zero() {
        assert!(mean(&white_noise(1, 20_000)).abs() < 0.05);
        let zero_seed = white_noise(0, 100);
        assert!(zero_seed.iter().any(|s| *s != zero_seed[0]));
    }

    #[test]
    fn low_pass_smooths_and_high_pass_keeps_the_hiss() {
        let noise = white_noise(3, 20_000);
        assert!(roughness(&low_pass(&noise, 350.0)) < roughness(&noise) / 2.0);
        assert!(roughness(&high_pass(&noise, 2500.0)) >= roughness(&noise) * 0.9);
        assert!(mean(&high_pass(&noise, 700.0)).abs() < 0.01);
    }

    #[test]
    fn ambience_loops_have_the_designed_length() {
        assert_eq!(rain_samples().len(), 44_100 * 2);
        assert_eq!(wind_samples().len(), 44_100 * 4);
        assert_eq!(sandstorm_samples().len(), 44_100 * 4);
    }

    #[test]
    fn ambience_peaks_match_their_target_volume() {
        for (name, samples, target) in ambience() {
            assert!((peak(&samples) - target).abs() < 1e-4, "{name}: {}", peak(&samples));
        }
    }

    #[test]
    fn ambience_is_centered_on_zero() {
        for (name, samples, _) in ambience() {
            assert!(mean(&samples).abs() < 0.02, "{name}: {}", mean(&samples));
        }
    }

    #[test]
    fn ambience_loops_without_a_click() {
        // Bước nhảy từ mẫu cuối về mẫu đầu không lớn hơn bước nhảy bình thường trong âm thanh
        for (name, samples, _) in ambience() {
            let seam = (samples[0] - samples[samples.len() - 1]).abs();
            assert!(seam <= 2.0 * max_step(&samples), "{name}: seam {seam}");
        }
    }

    #[test]
    fn rain_is_hissier_than_wind_and_sandstorm_is_in_between() {
        let (rain, wind, sand) = (
            roughness(&rain_samples()),
            roughness(&wind_samples()),
            roughness(&sandstorm_samples()),
        );
        assert!(rain > sand && sand > wind, "rain={rain}, sand={sand}, wind={wind}");
    }

    #[test]
    fn wind_and_sandstorm_swell_but_rain_stays_steady() {
        assert!(loudness_swing(&wind_samples()) > 1.5);
        assert!(loudness_swing(&sandstorm_samples()) > 1.5);
        assert!(loudness_swing(&rain_samples()) < 1.3);
    }

    #[test]
    fn ambience_wav_files_are_valid() {
        let sounds = [
            (rain_sound(), rain_samples().len()),
            (wind_sound(), wind_samples().len()),
            (sandstorm_sound(), sandstorm_samples().len()),
        ];
        for (wav, samples) in sounds {
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(u32_at(&wav, 40) as usize, samples * 2);
            assert_eq!(wav.len(), 44 + samples * 2);
        }
    }

    #[test]
    fn milestone_has_a_silent_gap_between_the_two_beeps() {
        let samples = concat(&[
            tone(Wave::Square, 880.0, 880.0, 70, 0.22),
            silence(30),
            tone(Wave::Square, 1320.0, 1320.0, 120, 0.22),
        ]);
        let first = tone(Wave::Square, 880.0, 880.0, 70, 0.22).len();
        let gap = silence(30).len();
        assert!(samples[first..first + gap].iter().all(|s| *s == 0.0));
        assert!(peak(&samples[first + gap..]) > 0.05);
    }
}
