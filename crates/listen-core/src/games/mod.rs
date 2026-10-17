//! Every game is a small module implementing [`Game`]. Adding a game means
//! one file here, one entry in [`ALL`], and its art in the web crate; the
//! touch gate, round machine, audio and settings are shared.

use crate::music::Sound;
use crate::rng::Rng;

mod birdfish;
mod echo;
mod loud;
mod notes;
mod play;
mod stairs;
mod tempo;
mod twins;

/// What a round asks.
#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    /// The target(s) to touch, in order. One entry for most games; Echo
    /// asks for a sequence.
    pub answer: Vec<u8>,
    /// What plays when the child presses Listen.
    pub sound: Sound,
    /// Shown and optionally spoken after a correct answer (Do Re Mi).
    pub name: Option<&'static str>,
    pub say: Option<&'static str>,
}

impl Question {
    pub fn simple(answer: u8, sound: Sound) -> Self {
        Question {
            answer: vec![answer],
            sound,
            name: None,
            say: None,
        }
    }
}

/// The adult-chosen settings a game reads when building a round.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Knobs {
    pub level: u8,
    /// Bar-note length in seconds (Do Re Mi, Echo, Free play).
    pub note_len: f32,
}

pub trait Game: Sync {
    /// Route token and storage key (`#notes`). Letters only.
    fn slug(&self) -> &'static str;
    /// Adult-facing name in the settings sheet.
    fn title(&self) -> &'static str;
    /// Level names, easiest first.
    fn levels(&self) -> &'static [&'static str];
    fn default_level(&self) -> u8 {
        0
    }
    /// Answer targets on screen at this level (ids are game-specific).
    fn targets(&self, level: u8) -> Vec<u8>;
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
    pub levels: &'static [&'static str],
    /// Returns the right target (0/1) and the question sound.
    pub ask: fn(level: u8, rng: &mut Rng) -> (u8, Sound),
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
    fn levels(&self) -> &'static [&'static str] {
        self.levels
    }
    fn targets(&self, _level: u8) -> Vec<u8> {
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

/// Clamp a stored level to what a game offers.
pub fn clamp_level(g: &dyn Game, level: u8) -> u8 {
    level.min(g.levels().len().saturating_sub(1) as u8)
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
            assert!(!g.levels().is_empty());
            assert!((g.default_level() as usize) < g.levels().len());
        }
    }

    #[test]
    fn every_level_of_every_game_builds_sane_rounds() {
        let mut rng = Rng::new(7);
        for g in ALL {
            for level in 0..g.levels().len() as u8 {
                let k = Knobs {
                    level,
                    note_len: 1.2,
                };
                let targets = g.targets(level);
                assert!(!targets.is_empty());
                let mut prev = None;
                for _ in 0..50 {
                    let q = g.question(k, &mut rng, prev.as_ref());
                    if !g.free() {
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
