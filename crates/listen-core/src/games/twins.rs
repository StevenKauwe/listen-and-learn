//! Same or Different: two notes; twin chicks or a chick and a ladybug.
//! Target 0 = same.

use super::{lit, Duo};
use crate::music::{Sound, Tone};
use crate::rng::Rng;

pub const TWINS: Duo = Duo {
    slug: "twins",
    title: "Same or Different",
    steps: &[
        "An octave",
        "A fifth",
        "A fourth",
        "A third",
        "A whole step",
        "A half step",
    ],
    ask,
    voices: [same, different],
    hint: same,
};

/// Semitones between the two notes when they differ, per step.
pub const GAPS: [&[u8]; 6] = [&[12], &[7], &[5], &[3, 4], &[2], &[1]];

fn ask(step: u8, rng: &mut Rng) -> (u8, Sound) {
    let base = rng.pick(&[57u8, 59, 60, 62, 64, 65, 67]);
    let same = rng.coin();
    let second = if same {
        base
    } else {
        let gap = rng.pick(GAPS[(step as usize).min(GAPS.len() - 1)]);
        if rng.coin() {
            base + gap
        } else {
            base - gap
        }
    };
    let a = if same { 0 } else { 1 };
    let s = Sound::of(vec![Tone::bar(base, 0.0, 0.9), Tone::bar(second, 1.0, 0.9)]);
    (a, lit(a, s))
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
    fn same_means_identical_and_different_means_the_step_gap() {
        let mut rng = Rng::new(2);
        for step in 0..GAPS.len() as u8 {
            for _ in 0..60 {
                let (a, s) = ask(step, &mut rng);
                let d = (s.tones[0].midi - s.tones[1].midi).abs() as u8;
                if a == 0 {
                    assert_eq!(d, 0)
                } else {
                    assert!(GAPS[step as usize].contains(&d))
                }
            }
        }
    }
}
