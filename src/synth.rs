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
