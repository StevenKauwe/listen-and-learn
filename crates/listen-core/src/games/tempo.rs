//! Turtle or Rabbit: the same little tune, slow or fast. Target 0 = turtle.

use super::Duo;
use crate::music::{Sound, Tone};
use crate::rng::Rng;

pub const TEMPO: Duo = Duo {
    slug: "tempo",
    title: "Turtle or Rabbit",
    levels: &["Very different", "Different", "Close"],
    ask,
    voices: [turtle, rabbit],
    hint: rabbit,
};

/// Seconds per note (slow, fast) per level.
pub const BEATS: [(f32, f32); 3] = [(0.8, 0.2), (0.62, 0.3), (0.5, 0.36)];
const TUNE: [u8; 5] = [67, 64, 67, 64, 60];

fn ask(level: u8, rng: &mut Rng) -> (u8, Sound) {
    let (slow, fast) = BEATS[(level as usize).min(2)];
    let is_slow = rng.coin();
    let beat = if is_slow { slow } else { fast };
    let tones = TUNE
        .iter()
        .enumerate()
        .map(|(i, &m)| Tone::bar(m, i as f32 * beat, 0.5))
        .collect();
    (if is_slow { 0 } else { 1 }, Sound::of(tones))
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
    fn slow_is_slower_than_fast() {
        for (slow, fast) in BEATS {
            assert!(slow / fast >= 1.35);
        }
    }
}
