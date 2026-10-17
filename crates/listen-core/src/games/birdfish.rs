//! Bird or Fish: high (bird, sky) or low (fish, water). Target 0 = bird.

use super::Duo;
use crate::music::{Sound, Tone};
use crate::rng::Rng;

pub const BIRDFISH: Duo = Duo {
    slug: "birdfish",
    title: "Bird or Fish",
    levels: &["Very far apart", "Two octaves", "One octave"],
    ask,
    voices: [bird, fish],
    hint,
};

fn ask(level: u8, rng: &mut Rng) -> (u8, Sound) {
    let high = rng.coin();
    // Low notes stay at or above A2 so phone speakers still carry them.
    let (hi, lo): (&[u8], &[u8]) = match level {
        0 => (&[81, 83, 84, 86], &[45, 47, 48, 50]),
        1 => (&[76, 77, 79], &[52, 53, 55]),
        _ => (&[67, 69], &[55, 57]),
    };
    let m = rng.pick(if high { hi } else { lo });
    let tones = if level >= 2 {
        // Close pitches get a middle-C reference first.
        vec![Tone::bar(60, 0.0, 0.8).gain(0.7), Tone::bar(m, 0.95, 0.9)]
    } else {
        vec![Tone::bar(m, 0.0, 0.8), Tone::bar(m, 0.5, 0.9)]
    };
    (if high { 0 } else { 1 }, Sound::of(tones))
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
    fn high_answers_are_higher_than_low_ones_at_every_level() {
        let mut rng = Rng::new(5);
        for level in 0..3u8 {
            let (mut min_hi, mut max_lo) = (200.0f32, 0.0f32);
            for _ in 0..100 {
                let (a, s) = ask(level, &mut rng);
                let m = s.tones.last().unwrap().midi;
                if a == 0 {
                    min_hi = min_hi.min(m)
                } else {
                    max_lo = max_lo.max(m)
                }
            }
            assert!(
                min_hi - max_lo >= 10.0,
                "level {level}: gap {}",
                min_hi - max_lo
            );
        }
    }
}
