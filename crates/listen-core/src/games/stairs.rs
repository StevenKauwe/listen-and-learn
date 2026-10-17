//! Up or Down: a short tune climbs or falls. Target 0 = up.

use super::Duo;
use crate::music::{Sound, Tone};
use crate::rng::Rng;

pub const STAIRS: Duo = Duo {
    slug: "stairs",
    title: "Up or Down",
    levels: &["Big jumps", "Five steps", "Three steps"],
    ask,
    voices: [up, down],
    hint: up,
};

const SHAPES: [&[u8]; 3] = [&[0, 4, 7, 12], &[0, 2, 4, 5, 7], &[0, 2, 4]];

fn ask(level: u8, rng: &mut Rng) -> (u8, Sound) {
    let shape = SHAPES[(level as usize).min(2)];
    let base = rng.pick(&[55u8, 57, 60, 62]);
    let going_up = rng.coin();
    let tones = (0..shape.len())
        .map(|i| {
            let off = if going_up {
                shape[i]
            } else {
                shape[shape.len() - 1 - i]
            };
            Tone::bar(base + off, i as f32 * 0.38, 0.55)
        })
        .collect();
    (if going_up { 0 } else { 1 }, Sound::of(tones))
}

fn run(notes: [u8; 3]) -> Sound {
    Sound::of(
        notes
            .iter()
            .enumerate()
            .map(|(i, &m)| Tone::bar(m, i as f32 * 0.1, 0.3))
            .collect(),
    )
}

fn up() -> Sound {
    run([67, 71, 74])
}

fn down() -> Sound {
    run([74, 71, 67])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_rises_and_down_falls() {
        let mut rng = Rng::new(4);
        for level in 0..3u8 {
            for _ in 0..40 {
                let (a, s) = ask(level, &mut rng);
                let rising = s.tones.windows(2).all(|w| w[1].midi > w[0].midi);
                let falling = s.tones.windows(2).all(|w| w[1].midi < w[0].midi);
                assert!(if a == 0 { rising } else { falling });
            }
        }
    }
}
