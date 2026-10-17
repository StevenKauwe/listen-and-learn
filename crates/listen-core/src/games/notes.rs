//! Do Re Mi: hear a note, touch its coloured bar. Targets are scale degrees.

use super::{Game, Knobs, Question};
use crate::music::{Sound, Tone, SCALE};
use crate::rng::Rng;

pub struct Notes;

/// Spec presets: Beginner Do/Mi/Sol, Standard Do/Re/Mi, Intermediate
/// Do–Sol, Advanced Do–Ti.
pub const SETS: [&[u8]; 4] = [
    &[0, 2, 4],
    &[0, 1, 2],
    &[0, 1, 2, 3, 4],
    &[0, 1, 2, 3, 4, 5, 6],
];

impl Game for Notes {
    fn slug(&self) -> &'static str {
        "notes"
    }
    fn title(&self) -> &'static str {
        "Do Re Mi"
    }
    fn levels(&self) -> &'static [&'static str] {
        &["Do Mi Sol", "Do Re Mi", "Do to Sol", "Do to Ti"]
    }
    fn default_level(&self) -> u8 {
        1
    }
    fn targets(&self, level: u8) -> Vec<u8> {
        SETS[(level as usize).min(3)].to_vec()
    }
    fn question(&self, k: Knobs, rng: &mut Rng, prev: Option<&Question>) -> Question {
        let set = self.targets(k.level);
        let last = prev.map(|q| q.answer[0]);
        // Never the same note twice in a row.
        let pool: Vec<u8> = set.iter().copied().filter(|d| Some(*d) != last).collect();
        let d = rng.pick(if pool.is_empty() { &set } else { &pool });
        let deg = SCALE[d as usize];
        Question {
            answer: vec![d],
            sound: Sound::of(vec![Tone::bar(deg.midi, 0.0, k.note_len)]),
            name: Some(deg.name),
            say: Some(deg.say),
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
        Sound::of(vec![
            Tone::bar(60, 0.0, 0.5),
            Tone::bar(62, 0.2, 0.5),
            Tone::bar(64, 0.4, 0.6),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_repeats_a_note_and_stays_in_the_preset() {
        let mut rng = Rng::new(3);
        for level in 0..4u8 {
            let k = Knobs {
                level,
                note_len: 1.2,
            };
            let mut prev: Option<Question> = None;
            for _ in 0..200 {
                let q = Notes.question(k, &mut rng, prev.as_ref());
                assert!(SETS[level as usize].contains(&q.answer[0]));
                if let Some(p) = &prev {
                    assert_ne!(p.answer, q.answer);
                }
                prev = Some(q);
            }
        }
    }

    #[test]
    fn question_and_answer_use_the_same_note_length() {
        let k = Knobs {
            level: 1,
            note_len: 1.6,
        };
        let q = Notes.question(k, &mut Rng::new(9), None);
        assert_eq!(q.sound.dur, 1.6);
        assert_eq!(Notes.answer_sound(k, q.answer[0]), q.sound);
    }
}
