//! Echo: copy a short pattern on the bars, Simon-style. With help on, the
//! bars light as they play; by ear otherwise. The ladder starts with the
//! sol–mi call every child already knows from playground chants.

use super::{Game, Knobs, Question};
use crate::music::{Sound, Tone, SCALE};
use crate::rng::Rng;

pub struct Echo;

pub struct Level {
    pub set: &'static [u8],
    pub len: usize,
}

pub const LEVELS: [Level; 5] = [
    Level {
        set: &[2, 4],
        len: 2,
    },
    Level {
        set: &[2, 4, 5],
        len: 2,
    },
    Level {
        set: &[2, 4, 5],
        len: 3,
    },
    Level {
        set: &[0, 2, 4, 5],
        len: 3,
    },
    Level {
        set: &[0, 1, 2, 4, 5],
        len: 4,
    },
];

/// Gap between pattern notes, seconds.
pub const STEP: f32 = 0.55;
const NOTE: f32 = 0.6;

fn lv(step: u8) -> &'static Level {
    &LEVELS[(step as usize).min(LEVELS.len() - 1)]
}

impl Game for Echo {
    fn slug(&self) -> &'static str {
        "echo"
    }
    fn title(&self) -> &'static str {
        "Echo"
    }
    fn steps(&self) -> &'static [&'static str] {
        &[
            "Sol Mi, 2 notes",
            "Sol Mi La, 2 notes",
            "Sol Mi La, 3 notes",
            "Do Mi Sol La, 3 notes",
            "Do Re Mi Sol La, 4 notes",
        ]
    }
    fn targets(&self, step: u8) -> Vec<u8> {
        lv(step).set.to_vec()
    }
    fn question(&self, k: Knobs, rng: &mut Rng, prev: Option<&Question>) -> Question {
        let l = lv(k.level);
        // With two bars there are only two patterns; allow repeats there
        // so the answer is not simply "the other one".
        let avoid_repeat = l.set.len() > 2;
        loop {
            let mut pat: Vec<u8> = Vec::with_capacity(l.len);
            while pat.len() < l.len {
                let d = rng.pick(l.set);
                if pat.last() != Some(&d) {
                    pat.push(d);
                }
            }
            if !avoid_repeat || prev.map(|p| p.answer != pat).unwrap_or(true) {
                let tones = pat
                    .iter()
                    .enumerate()
                    .map(|(i, &d)| Tone::bar(SCALE[d as usize].midi, i as f32 * STEP, NOTE).lit(d))
                    .collect();
                return Question {
                    answer: pat,
                    sound: Sound::of(tones),
                    name: None,
                    sing: None,
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
            Tone::bar(67, 0.0, 0.4),
            Tone::bar(64, 0.25, 0.5),
            Tone::bar(67, 0.75, 0.4).gain(0.5),
            Tone::bar(64, 1.0, 0.5).gain(0.5),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patterns_have_the_step_length_no_stutters_and_light_each_note() {
        let mut rng = Rng::new(11);
        for step in 0..LEVELS.len() as u8 {
            let q = Echo.question(
                Knobs {
                    level: step,
                    note_len: 1.2,
                },
                &mut rng,
                None,
            );
            assert_eq!(q.answer.len(), LEVELS[step as usize].len);
            assert!(q.answer.windows(2).all(|w| w[0] != w[1]));
            for (t, a) in q.sound.tones.iter().zip(&q.answer) {
                assert_eq!(t.light, Some(*a));
            }
        }
    }
}
