//! Turtle or Rabbit: the same little tune, slow or fast. Target 0 = turtle.
//! The ladder follows Bobin-Bègue & Provasi (2005): ±50 % around 600 ms
//! first, down to ±10 %.

use super::{lit, Duo};
use crate::music::{Sound, Tone};
use crate::rng::Rng;

pub const TEMPO: Duo = Duo {
    slug: "tempo",
    title: "Turtle or Rabbit",
    steps: &[
        "Three times slower",
        "Twice as slow",
        "±20 %",
        "±15 %",
        "±10 %",
    ],
    ask,
    voices: [turtle, rabbit],
    hint: rabbit,
};

/// Seconds per note (slow, fast) per step.
pub const BEATS: [(f32, f32); 5] = [
    (0.9, 0.3),
    (0.8, 0.4),
    (0.72, 0.48),
    (0.69, 0.51),
    (0.66, 0.54),
];
const TUNE: [u8; 5] = [67, 64, 67, 64, 60];

fn ask(step: u8, rng: &mut Rng) -> (u8, Sound) {
    let (slow, fast) = BEATS[(step as usize).min(BEATS.len() - 1)];
    let is_slow = rng.coin();
    let beat = if is_slow { slow } else { fast };
    let tones = TUNE
        .iter()
        .enumerate()
        .map(|(i, &m)| Tone::bar(m, i as f32 * beat, 0.5))
        .collect();
    let a = if is_slow { 0 } else { 1 };
    (a, lit(a, Sound::of(tones)))
}

fn turtle() -> Sound {
    Sound::of(vec![Tone::bar(48, 0.0, 0.5), Tone::bar(48, 0.55, 0.5)])
}

fn rabbit() -> Sound {
    Sound::of(
        [72u8, 74, 76, 79]
            .iter()
            .enumerate()
            .map(|(i, &m)| Tone::bar(m, i as f32 * 0.09, 0.22))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_is_slower_than_fast_and_the_ladder_closes() {
        let mut last = f32::MAX;
        for (slow, fast) in BEATS {
            let r = slow / fast;
            assert!(r >= 1.2 && r < last);
            last = r;
        }
    }
}
