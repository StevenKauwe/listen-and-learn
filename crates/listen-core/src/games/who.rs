//! Who's singing? The same little tune on the same notes, played by a
//! singer, a xylophone, a bell or a flute: only the timbre differs.
//! Preschoolers link timbres to pictures far more easily than melodies
//! (Creel 2016). Targets: 0 singer, 1 xylophone, 2 bell, 3 flute.

use super::{Game, Knobs, Question};
use crate::music::{Sound, Tone, Voice};
use crate::rng::Rng;

pub struct Who;

pub const SINGER: u8 = 0;
pub const XYLO: u8 = 1;
pub const BELL: u8 = 2;
pub const FLUTE: u8 = 3;

const SETS: [&[u8]; 3] = [
    &[SINGER, XYLO],
    &[SINGER, XYLO, BELL],
    &[SINGER, XYLO, BELL, FLUTE],
];
/// Sol, Mi and La around the children's singing range (D4–A4).
const TUNES: [[u8; 3]; 4] = [[67, 64, 67], [67, 64, 69], [64, 67, 64], [69, 67, 64]];
const GAP: f32 = 0.5;

fn set(step: u8) -> &'static [u8] {
    SETS[(step as usize).min(SETS.len() - 1)]
}

/// One note by instrument `who` (the singer sings "la").
pub fn note(who: u8, midi: u8, at: f32) -> Tone {
    let t = Tone::bar(midi, at, 0.55);
    match who {
        SINGER => t.voiced(Voice::Sing(5)).gain(0.9),
        BELL => t.voiced(Voice::Chime).gain(0.75),
        FLUTE => t.voiced(Voice::Flute).gain(0.85),
        _ => t,
    }
}

fn tune(who: u8, notes: &[u8]) -> Sound {
    Sound::of(
        notes
            .iter()
            .enumerate()
            .map(|(i, &m)| note(who, m, i as f32 * GAP).lit(who))
            .collect(),
    )
}

impl Game for Who {
    fn slug(&self) -> &'static str {
        "who"
    }
    fn title(&self) -> &'static str {
        "Who's singing?"
    }
    fn steps(&self) -> &'static [&'static str] {
        &["Singer or xylophone", "Add a bell", "Add a flute"]
    }
    fn targets(&self, step: u8) -> Vec<u8> {
        set(step).to_vec()
    }
    fn question(&self, k: Knobs, rng: &mut Rng, prev: Option<&Question>) -> Question {
        let set = set(k.level);
        let last = prev.map(|q| q.answer[0]);
        let pool: Vec<u8> = if set.len() >= 3 {
            set.iter().copied().filter(|d| Some(*d) != last).collect()
        } else {
            set.to_vec()
        };
        let who = rng.pick(&pool);
        let notes = TUNES[rng.below(TUNES.len())];
        Question::simple(who, tune(who, &notes))
    }
    fn answer_sound(&self, _k: Knobs, target: u8) -> Sound {
        let mut s = tune(target, &[67, 64]);
        for t in s.tones.iter_mut() {
            t.light = None;
        }
        s
    }
    fn hint(&self) -> Sound {
        Sound::of(vec![note(SINGER, 67, 0.0), note(XYLO, 64, 0.45)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_timbre_tells_them_apart() {
        let mut rng = Rng::new(8);
        for step in 0..SETS.len() as u8 {
            let k = Knobs {
                level: step,
                note_len: 1.2,
            };
            for _ in 0..40 {
                let q = Who.question(k, &mut rng, None);
                let who = q.answer[0];
                assert!(set(step).contains(&who));
                // Same pitches and timing whoever plays: the tune is one
                // of TUNES, spaced GAP apart, in the singing range.
                for (i, t) in q.sound.tones.iter().enumerate() {
                    assert!((62.0..=69.0).contains(&t.midi));
                    assert_eq!(t.at, i as f32 * GAP);
                    assert_eq!(t.light, Some(who));
                }
            }
        }
    }

    #[test]
    fn each_instrument_has_its_own_voice() {
        let v: Vec<Voice> = [SINGER, XYLO, BELL, FLUTE]
            .iter()
            .map(|&w| note(w, 67, 0.0).voice)
            .collect();
        for i in 0..v.len() {
            assert!(v[i + 1..].iter().all(|o| *o != v[i]));
        }
    }
}
