//! Mouse or Elephant: the same tune, quiet or loud. Target 0 = mouse.

use super::{lit, Duo};
use crate::music::{Sound, Tone};
use crate::rng::Rng;

pub const LOUD: Duo = Duo {
    slug: "loud",
    title: "Mouse or Elephant",
    steps: &["24 dB", "18 dB", "12 dB", "8 dB", "6 dB"],
    ask,
    voices: [mouse, elephant],
    hint,
};

/// Loudness gap in decibels per step.
pub const GAP_DB: [f32; 5] = [24.0, 18.0, 12.0, 8.0, 6.0];

fn ask(step: u8, rng: &mut Rng) -> (u8, Sound) {
    let quiet = rng.coin();
    let g = if quiet {
        10f32.powf(-GAP_DB[(step as usize).min(GAP_DB.len() - 1)] / 20.0)
    } else {
        1.0
    };
    let tones = [60u8, 64, 67]
        .iter()
        .enumerate()
        .map(|(i, &m)| Tone::bar(m, i as f32 * 0.4, 0.6).gain(g))
        .collect();
    let a = if quiet { 0 } else { 1 };
    (a, lit(a, Sound::of(tones)))
}

fn mouse() -> Sound {
    Sound::of(vec![
        Tone::bar(91, 0.0, 0.15).gain(0.12),
        Tone::bar(93, 0.12, 0.15).gain(0.12),
    ])
}

fn elephant() -> Sound {
    Sound::of(vec![
        Tone::bar(43, 0.0, 0.7),
        Tone::bar(50, 0.05, 0.6).gain(0.5),
    ])
}

fn hint() -> Sound {
    let mut t = mouse().tones;
    t.extend(elephant().tones.into_iter().map(|mut x| {
        x.at += 0.4;
        x
    }));
    Sound::of(t)
}
