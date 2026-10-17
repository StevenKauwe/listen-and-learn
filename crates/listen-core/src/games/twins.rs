//! Same or Different: two notes; twin chicks or a chick and a ladybug.
//! Target 0 = same.

use super::Duo;
use crate::music::{Sound, Tone};
use crate::rng::Rng;

pub const TWINS: Duo = Duo {
    slug: "twins",
    title: "Same or Different",
    levels: &["Very different", "Different", "Close"],
    ask,
    voices: [same, different],
    hint: same,
};

/// Semitones between the two notes when they differ, per level.
pub const GAPS: [&[u8]; 3] = [&[12], &[5, 7], &[2]];

fn ask(level: u8, rng: &mut Rng) -> (u8, Sound) {
    let base = rng.pick(&[57u8, 59, 60, 62, 64, 65, 67]);
    let same = rng.coin();
    let second = if same {
        base
    } else {
        let gap = rng.pick(GAPS[(level as usize).min(2)]);
        if rng.coin() {
            base + gap
        } else {
            base - gap
        }
    };
    let s = Sound::of(vec![Tone::bar(base, 0.0, 0.9), Tone::bar(second, 1.0, 0.9)]);
    (if same { 0 } else { 1 }, s)
}

fn same() -> Sound {
    Sound::of(vec![Tone::bar(76, 0.0, 0.25), Tone::bar(76, 0.2, 0.3)])
}

fn different() -> Sound {
    Sound::of(vec![Tone::bar(76, 0.0, 0.25), Tone::bar(69, 0.2, 0.3)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_means_identical_and_different_means_the_level_gap() {
        let mut rng = Rng::new(2);
        for level in 0..3u8 {
            for _ in 0..60 {
                let (a, s) = ask(level, &mut rng);
                let d = (s.tones[0].midi - s.tones[1].midi).abs() as u8;
                if a == 0 {
                    assert_eq!(d, 0)
                } else {
                    assert!(GAPS[level as usize].contains(&d))
                }
            }
        }
    }
}
