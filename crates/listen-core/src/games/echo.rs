//! Echo: copy a short pattern on the bars. Easy levels light the bars as
//! they play; harder levels are by ear only.

use super::{Game, Knobs, Question};
use crate::music::{Sound, Tone, SCALE};
use crate::rng::Rng;

pub struct Echo;

struct Level {
    set: &'static [u8],
    len: usize,
    lights: bool,
}

const LEVELS: [Level; 4] = [
    Level {
        set: &[0, 2, 4],
        len: 2,
        lights: true,
    },
    Level {
        set: &[0, 2, 4],
        len: 3,
        lights: true,
    },
    Level {
        set: &[0, 2, 4],
        len: 3,
        lights: false,
    },
    Level {
        set: &[0, 1, 2, 3, 4],
        len: 4,
        lights: false,
    },
];

/// Gap between pattern notes, seconds.
pub const STEP: f32 = 0.55;
const NOTE: f32 = 0.6;

fn lv(level: u8) -> &'static Level {
    &LEVELS[(level as usize).min(LEVELS.len() - 1)]
}

impl Game for Echo {
    fn slug(&self) -> &'static str {
        "echo"
    }
    fn title(&self) -> &'static str {
        "Echo"
    }
    fn levels(&self) -> &'static [&'static str] {
        &[
            "Two notes, lights",
            "Three notes, lights",
            "Three notes, by ear",
            "Four notes, by ear",
        ]
    }
    fn targets(&self, level: u8) -> Vec<u8> {
        lv(level).set.to_vec()
    }
    fn question(&self, k: Knobs, rng: &mut Rng, prev: Option<&Question>) -> Question {
        let l = lv(k.level);
        loop {
            let mut pat: Vec<u8> = Vec::with_capacity(l.len);
            while pat.len() < l.len {
                let d = rng.pick(l.set);
                if pat.last() != Some(&d) {
                    pat.push(d);
                }
            }
            if prev.map(|p| p.answer != pat).unwrap_or(true) {
                let tones = pat
                    .iter()
                    .enumerate()
                    .map(|(i, &d)| {
                        let t = Tone::bar(SCALE[d as usize].midi, i as f32 * STEP, NOTE);
                        if l.lights {
                            t.lit(d)
                        } else {
                            t
                        }
                    })
                    .collect();
                return Question {
                    answer: pat,
                    sound: Sound::of(tones),
                    name: None,
                    say: None,
                };
            }
        }
    }
    fn answer_sound(&self, _k: Knobs, target: u8) -> Sound {
        Sound::of(vec![Tone::bar(
            SCALE[(target as usize).min(7)].midi,
            0.0,
            NOTE,
        )])
    }
    fn hint(&self) -> Sound {
        Sound::of(vec![
            Tone::bar(60, 0.0, 0.4),
            Tone::bar(67, 0.22, 0.5),
            Tone::bar(60, 0.7, 0.4).gain(0.5),
            Tone::bar(67, 0.92, 0.5).gain(0.5),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns_have_the_level_length_and_no_stutters() {
        let mut rng = Rng::new(11);
        for level in 0..4u8 {
            let q = Echo.question(
                Knobs {
                    level,
                    note_len: 1.2,
                },
                &mut rng,
                None,
            );
            assert_eq!(q.answer.len(), LEVELS[level as usize].len);
            assert!(q.answer.windows(2).all(|w| w[0] != w[1]));
            let lit = q.sound.tones.iter().all(|t| t.light.is_some());
            assert_eq!(lit, LEVELS[level as usize].lights);
        }
    }
}
