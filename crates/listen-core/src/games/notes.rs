//! Do Re Mi: hear a note, touch its coloured bar. Targets are scale
//! degrees. The ladder follows the Kodály order: sol–mi first, then la,
//! do, re (pentatonic), and fa and ti last.

use super::{Game, Knobs, Question};
use crate::music::{Sound, Tone, SCALE};
use crate::rng::Rng;

pub struct Notes;

/// Bars per step, lowest first (7 = high Do).
pub const SETS: [&[u8]; 5] = [
    &[2, 4],
    &[2, 4, 5],
    &[0, 2, 4, 5],
    &[0, 1, 2, 4, 5],
    &[0, 1, 2, 3, 4, 5, 6, 7],
];

/// Pause between the bar note and its sung name.
pub const SING_AT: f32 = 0.45;

fn set(step: u8) -> &'static [u8] {
    SETS[(step as usize).min(SETS.len() - 1)]
}

impl Game for Notes {
    fn slug(&self) -> &'static str {
        "notes"
    }
    fn title(&self) -> &'static str {
        "Do Re Mi"
    }
    fn steps(&self) -> &'static [&'static str] {
        &[
            "Sol Mi",
            "Sol Mi La",
            "Do Mi Sol La",
            "Do Re Mi Sol La",
            "Do to high Do",
        ]
    }
    fn targets(&self, step: u8) -> Vec<u8> {
        set(step).to_vec()
    }
    fn question(&self, k: Knobs, rng: &mut Rng, prev: Option<&Question>) -> Question {
        let set = set(k.level);
        let last = prev.map(|q| q.answer[0]);
        // With three or more bars, never the same note twice running. With
        // two, strict alternation would be a pattern to learn instead of
        // the pitch; the round caps runs at three instead.
        let pool: Vec<u8> = if set.len() >= 3 {
            set.iter().copied().filter(|d| Some(*d) != last).collect()
        } else {
            set.to_vec()
        };
        let d = rng.pick(&pool);
        let deg = SCALE[d as usize];
        Question {
            answer: vec![d],
            sound: Sound::of(vec![Tone::bar(deg.midi, 0.0, k.note_len).lit(d)]),
            name: Some(deg.name),
            sing: Some((d, deg.midi)),
        }
    }
    fn answer_sound(&self, k: Knobs, target: u8) -> Sound {
        Sound::of(vec![Tone::bar(
            SCALE[(target as usize).min(7)].midi,
            0.0,
            k.note_len,
        )])
    }
    fn hint(&self) -> Sound {
        Sound::of(vec![Tone::bar(67, 0.0, 0.5), Tone::bar(64, 0.3, 0.6)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stays_in_the_set_and_never_repeats_with_three_or_more_bars() {
        let mut rng = Rng::new(3);
        for level in 0..SETS.len() as u8 {
            let k = Knobs {
                level,
                note_len: 1.2,
            };
            let mut prev: Option<Question> = None;
            let mut seen = vec![];
            for _ in 0..200 {
                let q = Notes.question(k, &mut rng, prev.as_ref());
                assert!(SETS[level as usize].contains(&q.answer[0]));
                if let (Some(p), true) = (&prev, SETS[level as usize].len() >= 3) {
                    assert_ne!(p.answer, q.answer);
                }
                seen.push(q.answer[0]);
                prev = Some(q);
            }
            for d in SETS[level as usize] {
                assert!(seen.contains(d), "every bar gets asked");
            }
        }
    }

    #[test]
    fn kodaly_order_sol_mi_first_then_la() {
        assert_eq!(SETS[0], &[2, 4]);
        assert_eq!(SETS[1], &[2, 4, 5]);
        // Fa (3) and Ti (6) only arrive on the last step.
        assert!(SETS[..4].iter().all(|s| !s.contains(&3) && !s.contains(&6)));
    }

    #[test]
    fn question_lights_its_bar_and_matches_the_answer_sound() {
        let k = Knobs {
            level: 1,
            note_len: 1.6,
        };
        let q = Notes.question(k, &mut Rng::new(9), None);
        assert_eq!(q.sound.dur, 1.6);
        assert_eq!(q.sound.tones[0].light, Some(q.answer[0]));
        let mut plain = q.sound.clone();
        plain.tones[0].light = None;
        assert_eq!(Notes.answer_sound(k, q.answer[0]), plain);
    }
}
