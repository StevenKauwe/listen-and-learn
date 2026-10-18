//! Every game is a small module implementing [`Game`]. Adding a game means
//! one file here, one entry in [`ALL`], and its art in the web crate; the
//! touch gate, round machine, audio and settings are shared.

use crate::music::Sound;
use crate::rng::Rng;

pub mod birdfish;
pub mod drum;
pub mod echo;
pub mod loud;
pub mod notes;
pub mod play;
pub mod stairs;
pub mod tempo;
pub mod twins;
pub mod who;

/// What a round asks.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    /// The target(s) to touch, in order. One entry for most games; Echo
    /// asks for a sequence.
    pub answer: Vec<u8>,
    /// What plays when the child presses Listen.
    pub sound: Sound,
    /// Shown after a correct answer (Do Re Mi).
    pub name: Option<&'static str>,
    /// Sung after a correct answer when names are on: (degree, MIDI note).
    pub sing: Option<(u8, u8)>,
}

impl Question {
    pub fn simple(answer: u8, sound: Sound) -> Self {
        Question {
            answer: vec![answer],
            sound,
            name: None,
            sing: None,
        }
    }
}

/// The adult-chosen settings a game reads when building a round.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Knobs {
    /// The rung of the game's difficulty ladder.
    pub level: u8,
    /// Bar-note length in seconds (Do Re Mi, Echo, Free play).
    pub note_len: f32,
}

pub trait Game: Sync {
    /// Route token and storage key (`#notes`). Letters only.
    fn slug(&self) -> &'static str;
    /// Adult-facing name in the settings sheet.
    fn title(&self) -> &'static str;
    /// The difficulty ladder, easiest first (calib.rs picks a band per age).
    fn steps(&self) -> &'static [&'static str];
    /// Answer targets on screen at this step (ids are game-specific).
    fn targets(&self, step: u8) -> Vec<u8>;
    /// A new question. `prev` lets a game avoid immediate repeats.
    fn question(&self, k: Knobs, rng: &mut Rng, prev: Option<&Question>) -> Question;
    /// What touching `target` sounds like.
    fn answer_sound(&self, k: Knobs, target: u8) -> Sound;
    /// Short sound when its home tile is tapped.
    fn hint(&self) -> Sound;
    /// Free play: no questions, targets sound at contact.
    fn free(&self) -> bool {
        false
    }
    /// Drum along: a beat game with its own loop (games/drum.rs).
    fn beat(&self) -> bool {
        false
    }
}

/// Home-screen order.
pub static ALL: &[&dyn Game] = &[
    &notes::Notes,
    &birdfish::BIRDFISH,
    &twins::TWINS,
    &stairs::STAIRS,
    &echo::Echo,
    &tempo::TEMPO,
    &loud::LOUD,
    &who::Who,
    &drum::Drum,
    &play::Play,
];

pub fn by_slug(slug: &str) -> Option<(usize, &'static dyn Game)> {
    ALL.iter()
        .enumerate()
        .find(|(_, g)| g.slug() == slug)
        .map(|(i, g)| (i, *g))
}

/// A two-picture game: target 0 or 1. Most games are one of these, so a
/// new one is a single constant.
pub struct Duo {
    pub slug: &'static str,
    pub title: &'static str,
    pub steps: &'static [&'static str],
    /// Returns the right target (0/1) and the question sound, whose tones
    /// carry `light` = the right target (the help shows it).
    pub ask: fn(step: u8, rng: &mut Rng) -> (u8, Sound),
    /// The sound each picture makes when touched.
    pub voices: [fn() -> Sound; 2],
    pub hint: fn() -> Sound,
}

impl Game for Duo {
    fn slug(&self) -> &'static str {
        self.slug
    }
    fn title(&self) -> &'static str {
        self.title
    }
    fn steps(&self) -> &'static [&'static str] {
        self.steps
    }
    fn targets(&self, _step: u8) -> Vec<u8> {
        vec![0, 1]
    }
    fn question(&self, k: Knobs, rng: &mut Rng, _prev: Option<&Question>) -> Question {
        let (a, s) = (self.ask)(k.level, rng);
        Question::simple(a, s)
    }
    fn answer_sound(&self, _k: Knobs, target: u8) -> Sound {
        (self.voices[(target & 1) as usize])()
    }
    fn hint(&self) -> Sound {
        (self.hint)()
    }
}

/// Light every tone of a two-picture question with its answer.
pub fn lit(answer: u8, mut s: Sound) -> Sound {
    for t in s.tones.iter_mut() {
        t.light = Some(answer);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_unique_bare_tokens() {
        for (i, g) in ALL.iter().enumerate() {
            assert!(
                g.slug().chars().all(|c| c.is_ascii_lowercase()),
                "{}",
                g.slug()
            );
            assert!(ALL[i + 1..].iter().all(|o| o.slug() != g.slug()));
            assert!(!g.steps().is_empty());
        }
    }

    #[test]
    fn every_level_of_every_game_builds_sane_rounds() {
        let mut rng = Rng::new(7);
        for g in ALL {
            for level in 0..g.steps().len() as u8 {
                let k = Knobs {
                    level,
                    note_len: 1.2,
                };
                let targets = g.targets(level);
                assert!(!targets.is_empty());
                let mut prev = None;
                for _ in 0..50 {
                    let q = g.question(k, &mut rng, prev.as_ref());
                    if !g.free() && !g.beat() {
                        assert!(q.sound.dur > 0.2, "{} silent question", g.slug());
                        assert!(q.answer.iter().all(|a| targets.contains(a)));
                    }
                    for t in &q.sound.tones {
                        assert!((24.0..=108.0).contains(&t.midi));
                        assert!(t.gain > 0.0 && t.gain <= 1.0);
                    }
                    prev = Some(q);
                }
                for t in targets {
                    assert!(g.answer_sound(k, t).dur > 0.0);
                }
            }
            assert!(g.hint().dur > 0.0);
        }
    }
}
