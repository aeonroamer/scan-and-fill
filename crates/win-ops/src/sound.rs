use std::f32::consts::TAU;
use std::sync::OnceLock;

use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};
use windows::core::PCWSTR;

const RATE: u32 = 22_050;
const VOLUME: f32 = 0.12;
const FADE_SECS: f32 = 0.008;

static REJECT: OnceLock<Vec<u8>> = OnceLock::new();

pub fn play_reject() {
    let wav = REJECT.get_or_init(|| wav(&[(520.0, 0.07), (390.0, 0.11)]));
    unsafe {
        let _ = PlaySoundW(
            PCWSTR(wav.as_ptr() as *const u16),
            None,
            SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
        );
    }
}

fn wav(notes: &[(f32, f32)]) -> Vec<u8> {
    let fade = (RATE as f32 * FADE_SECS) as usize;
    let mut samples = Vec::new();
    for &(freq, secs) in notes {
        let count = (RATE as f32 * secs) as usize;
        for i in 0..count {
            let edge = i.min(count - 1 - i) as f32 / fade as f32;
            let t = i as f32 / RATE as f32;
            let value = (TAU * freq * t).sin() * edge.min(1.0) * VOLUME;
            samples.push((value * i16::MAX as f32) as i16);
        }
    }

    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}
