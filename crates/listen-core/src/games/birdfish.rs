//! Bird or Fish: high (bird, sky) or low (fish, water). Target 0 = bird.
//! The ladder closes the gap from three octaves to a fifth either side of
//! a middle-C reference.

use super::{lit, Duo};
use crate::music::{Sound, Tone};
use crate::rng::Rng;

pub const BIRDFISH: Duo = Duo {
    slug: "birdfish",
    title: "Bird or Fish",
    steps: &[
        "Very far apart",
        "Two octaves",
        "An octave from middle C",
        "A fifth from middle C",
    ],
    ask,
    voices: [bird, fish],
    hint,
};

/// High and low notes per step. Low notes stay at or above A2 (110 Hz)
/// so phone speakers still carry them.
const NOTES: [(&[u8], &[u8]); 4] = [
    (&[81, 83, 84, 86], &[45, 47, 48, 50]),
    (&[76, 77, 79], &[52, 53, 55]),
    (&[70, 72], &[48, 50]),
    (&[65, 67], &[53, 55]),
];

/// Steps from this one on play middle C first as a reference.
pub const REFERENCE_FROM: u8 = 2;

fn notes(step: u8) -> (&'static [u8], &'static [u8]) {
    NOTES[(step as usize).min(NOTES.len() - 1)]
}

/// Semitones between the lowest high note and the highest low note.
pub fn gap(step: u8) -> u8 {
    let (hi, lo) = notes(step);
    hi.iter().min().unwrap() - lo.iter().max().unwrap()
}

fn ask(step: u8, rng: &mut Rng) -> (u8, Sound) {
    let high = rng.coin();
    let (hi, lo) = notes(step);
    let m = rng.pick(if high { hi } else { lo });
    let a = if high { 0 } else { 1 };
    let s = if step >= REFERENCE_FROM {
        let mut s = lit(a, Sound::of(vec![Tone::bar(m, 0.95, 0.9)]));
        s.tones.insert(0, Tone::bar(60, 0.0, 0.8).gain(0.7));
        s.dur = s.dur.max(0.8);
        s
    } else {
        lit(
            a,
            Sound::of(vec![Tone::bar(m, 0.0, 0.8), Tone::bar(m, 0.5, 0.9)]),
        )
    };
    (a, s)
}

fn bird() -> Sound {
    Sound::of(vec![
        Tone::bar(96, 0.0, 0.2).gain(0.45),
        Tone::bar(100, 0.13, 0.25).gain(0.45),
    ])
}

fn fish() -> Sound {
    Sound::of(vec![
        Tone::bar(45, 0.0, 0.45).gain(0.9),
        Tone::bar(43, 0.22, 0.5).gain(0.8),
    ])
}

fn hint() -> Sound {
    let mut t = bird().tones;
    t.extend(fish().tones.into_iter().map(|mut x| {
        x.at += 0.45;
        x
    }));
    Sound::of(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_answers_are_higher_than_low_ones_by_the_step_gap() {
        let mut rng = Rng::new(5);
        let mut last_gap = u8::MAX;
        for step in 0..4u8 {
            let (mut min_hi, mut max_lo) = (200.0f32, 0.0f32);
            for _ in 0..100 {
                let (a, s) = ask(step, &mut rng);
                let m = s.tones.last().unwrap().midi;
                if a == 0 {
                    min_hi = min_hi.min(m)
                } else {
                    max_lo = max_lo.max(m)
                }
                assert!(m >= 45.0, "nothing below A2");
            }
            assert_eq!((min_hi - max_lo) as u8, gap(step));
            assert!(gap(step) <= last_gap, "the ladder only gets closer");
            last_gap = gap(step);
        }
        assert!(gap(3) >= 10, "even the top step stays a clear gap");
    }

    #[test]
    fn the_reference_note_is_never_lit() {
        let (_, s) = ask(REFERENCE_FROM, &mut Rng::new(2));
        assert_eq!(s.tones[0].midi, 60.0);
        assert_eq!(s.tones[0].light, None);
        assert!(s.tones[1].light.is_some());
    }
}
