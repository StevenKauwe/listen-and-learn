//! Free play: a xylophone with no wrong answers. Bars sound at contact.

use super::{Game, Knobs, Question};
use crate::music::{Sound, Tone, SCALE};
use crate::rng::Rng;

pub struct Play;

impl Game for Play {
    fn slug(&self) -> &'static str {
        "play"
    }
    fn title(&self) -> &'static str {
        "Free play"
    }
    fn steps(&self) -> &'static [&'static str] {
        // Pentatonic first: any order of these notes sounds pleasant.
        &["Six bars (pentatonic)", "Eight bars (Do to Do)"]
    }
    fn targets(&self, step: u8) -> Vec<u8> {
        if step == 0 {
            vec![0, 1, 2, 4, 5, 7]
        } else {
            (0..8).collect()
        }
    }
    fn question(&self, _k: Knobs, _rng: &mut Rng, _prev: Option<&Question>) -> Question {
        Question {
            answer: vec![],
            sound: Sound::default(),
            name: None,
            sing: None,
        }
    }
    fn answer_sound(&self, k: Knobs, target: u8) -> Sound {
        Sound::of(vec![Tone::bar(
            SCALE[(target as usize).min(7)].midi,
            0.0,
            k.note_len.max(1.2),
        )])
    }
    fn hint(&self) -> Sound {
        let run = [60u8, 62, 64, 67, 69, 72];
        Sound::of(
            run.iter()
                .enumerate()
                .map(|(i, &m)| Tone::bar(m, i as f32 * 0.09, 0.5))
                .collect(),
        )
    }
    fn free(&self) -> bool {
        true
    }
}
