//! Difficulty calibration: one age choice sets every game. The numbers
//! and the reasons behind them are in docs/research.md; the tests at the
//! bottom pin the research-derived limits (age 2 hears only maximal
//! contrasts, never sees more than two choices, and is shown the answer
//! first).

/// The one simple control: the child's age.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Age {
    Two,
    Three,
    Four,
    Five,
}

impl Age {
    pub const ALL: [Age; 4] = [Age::Two, Age::Three, Age::Four, Age::Five];

    pub fn label(self) -> &'static str {
        match self {
            Age::Two => "2",
            Age::Three => "3",
            Age::Four => "4",
            Age::Five => "5+",
        }
    }

    pub fn years(self) -> u8 {
        self as u8 + 2
    }

    pub fn from_years(y: u8) -> Age {
        match y {
            0..=2 => Age::Two,
            3 => Age::Three,
            4 => Age::Four,
            _ => Age::Five,
        }
    }

    /// One line for grown-ups under the age chips.
    pub fn blurb(self) -> &'static str {
        match self {
            Age::Two => "Two choices, huge differences, the answer shown first. Short sets.",
            Age::Three => "Big differences that slowly get closer. Lights help with notes.",
            Age::Four => "Closer sounds and three or four choices. Hints fade out.",
            Age::Five => "Fine differences, by ear, hints only when stuck.",
        }
    }

    /// Drum along counts on-beat drums in a row from 4: many 3–5-year-olds
    /// cannot keep a beat yet (docs/research.md), so younger children just
    /// drum, with nothing counted and nothing ever wrong.
    pub fn streak(self) -> bool {
        self >= Age::Four
    }

    /// Right answers in a set before a rest (0 = no rest).
    pub fn set_len(self) -> u8 {
        match self {
            Age::Two => 6,
            Age::Three => 8,
            Age::Four => 10,
            Age::Five => 12,
        }
    }
}

/// How strongly a round shows its answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Help {
    /// By ear only.
    None,
    /// The answer lights up while the question plays.
    Light,
    /// Lights up, glows, and the other choices are dimmed and cannot be
    /// wrong (they only make their sound): errorless.
    Show,
}

/// Per-game difficulty, everything the "More" panel exposes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tune {
    /// Easiest and hardest step of the game's ladder; the game adapts
    /// between them (closer after first-try successes, back after misses).
    pub lo: u8,
    pub hi: u8,
    /// The first this-many right answers of a visit are shown (Help::Show).
    pub show: u8,
    /// After those, light the answer while the question plays.
    pub light: bool,
    /// Glow the answer after this many misses (0 = never).
    pub glow_miss: u8,
    /// Glow the answer after this many seconds without a touch (0 = never).
    pub glow_wait: u8,
    /// Replay the question after this many misses (0 = never).
    pub replay: u8,
}

/// First-try rights in a row before the game steps one rung harder.
pub const STEP_UP_AFTER: u32 = 3;
/// Misses on one question that step it one rung easier.
pub const STEP_DOWN_MISSES: u32 = 2;
/// Questions in a row that needed that many misses before the answer is
/// shown again for one round.
pub const RESHOW_AFTER: u32 = 2;

/// Free play: bars that may sound at once, at every age. A hand's worth:
/// two-year-olds touch with several fingers far more than older children
/// (docs/research.md), and with one at a time a resting finger silences
/// every other touch.
pub const NOTES_AT_ONCE: u8 = 5;

/// The preset for one game at one age. `slug` names the game.
pub fn preset(age: Age, slug: &str) -> Tune {
    use Age::*;
    // Ladder bands per age (indices into each game's steps; see games/).
    let (lo, hi) = match (slug, age) {
        ("notes", Two) => (0, 0),
        ("notes", Three) => (1, 1),
        ("notes", Four) => (2, 2),
        ("notes", Five) => (3, 3),
        ("birdfish" | "stairs", Two) => (0, 0),
        ("birdfish" | "stairs", Three) => (0, 1),
        ("birdfish" | "stairs", Four) => (0, 2),
        ("birdfish" | "stairs", Five) => (1, 3),
        ("twins", Two) => (0, 0),
        ("twins", Three) => (0, 1),
        ("twins", Four) => (0, 2),
        ("twins", Five) => (1, 4),
        ("tempo" | "loud", Two) => (0, 0),
        ("tempo" | "loud", Three) => (0, 1),
        ("tempo" | "loud", Four) => (0, 3),
        ("tempo" | "loud", Five) => (1, 4),
        ("echo", Two) => (0, 0),
        ("echo", Three) => (1, 2),
        ("echo", Four) => (3, 3),
        ("echo", Five) => (3, 4),
        ("who", Two) => (0, 0),
        ("who", Three) => (1, 1),
        ("who", _) => (2, 2),
        ("drum", Two | Three) => (0, 0),
        ("drum", Four) => (1, 1),
        ("drum", Five) => (2, 2),
        ("play", Two | Three) => (0, 0),
        ("play", _) => (1, 1),
        _ => (0, 0),
    };
    let (show, light, glow_miss, glow_wait, replay) = match age {
        Two => (3, true, 1, 4, 2),
        Three => (2, slug == "notes" || slug == "echo", 2, 6, 2),
        Four => (1, false, 3, 0, 2),
        Five => (0, false, 0, 0, 3),
    };
    Tune {
        lo,
        hi,
        show,
        light,
        glow_miss,
        glow_wait,
        replay,
    }
}

impl Tune {
    /// Clamp to a game with `steps` ladder rungs.
    pub fn clamped(mut self, steps: usize) -> Tune {
        let last = steps.saturating_sub(1) as u8;
        self.lo = self.lo.min(last);
        self.hi = self.hi.clamp(self.lo, last);
        self.show = self.show.min(9);
        self.glow_miss = self.glow_miss.min(9);
        self.glow_wait = self.glow_wait.min(30);
        self.replay = self.replay.min(9);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::{birdfish, tempo, twins, ALL};
    use crate::rng::Rng;
    use crate::Knobs;

    fn band(age: Age, slug: &str) -> std::ops::RangeInclusive<u8> {
        let t = preset(age, slug);
        t.lo..=t.hi
    }

    #[test]
    fn age_two_hears_only_maximal_pitch_contrasts() {
        // Two-note comparisons are at least an octave apart at age 2.
        for step in band(Age::Two, "twins") {
            assert!(twins::GAPS[step as usize]
                .iter()
                .all(|&g| g >= AGE2_MIN_GAP));
        }
        for step in band(Age::Two, "birdfish") {
            assert!(birdfish::gap(step) >= AGE2_MIN_GAP);
        }
        // Up or Down spans a whole octave at age 2.
        for step in band(Age::Two, "stairs") {
            let s = crate::games::stairs::SHAPES[step as usize];
            assert!(s[s.len() - 1] - s[0] >= AGE2_MIN_GAP);
        }
    }

    /// Smallest pitch difference a 2-year-old is ever asked to judge.
    const AGE2_MIN_GAP: u8 = 12;

    #[test]
    fn age_two_never_sees_more_than_two_choices() {
        for g in ALL {
            if g.free() {
                continue;
            }
            for step in band(Age::Two, g.slug()) {
                assert!(g.targets(step).len() <= 2, "{} step {step}", g.slug());
            }
        }
    }

    #[test]
    fn age_two_tempo_and_loudness_are_huge() {
        let (slow, fast) = tempo::BEATS[preset(Age::Two, "tempo").hi as usize];
        assert!(slow >= 3.0 * fast - 1e-6);
        let db = crate::games::loud::GAP_DB[preset(Age::Two, "loud").hi as usize];
        assert!(db >= 24.0);
    }

    #[test]
    fn age_two_is_shown_first_lit_and_rescued_quickly() {
        for g in ALL {
            let t = preset(Age::Two, g.slug());
            assert!(t.show >= 2 && t.light, "{}", g.slug());
            assert!((1..=1).contains(&t.glow_miss));
            assert!(
                (1..=5).contains(&t.glow_wait),
                "delayed prompt, not instant"
            );
            assert_eq!(t.lo, t.hi, "no adaptive tightening at age 2");
        }
        assert!(Age::Two.set_len() <= 8);
    }

    #[test]
    fn bands_get_harder_with_age_and_help_fades() {
        for g in ALL {
            let steps = g.steps().len() as u8;
            for w in Age::ALL.windows(2) {
                let (a, b) = (preset(w[0], g.slug()), preset(w[1], g.slug()));
                assert!(b.lo >= a.lo && b.hi >= a.hi, "{} {:?}", g.slug(), w);
                assert!(b.show <= a.show);
                assert!(a.light >= b.light);
                assert!(b.hi < steps && b.lo <= b.hi);
            }
        }
    }

    #[test]
    fn every_preset_question_respects_its_band() {
        let mut rng = Rng::new(17);
        for g in ALL {
            for age in Age::ALL {
                let t = preset(age, g.slug());
                for step in t.lo..=t.hi {
                    let k = Knobs {
                        level: step,
                        note_len: 1.2,
                    };
                    let q = g.question(k, &mut rng, None);
                    let targets = g.targets(step);
                    assert!(q.answer.iter().all(|a| targets.contains(a)));
                }
            }
        }
    }

    #[test]
    fn tune_clamps_into_the_ladder() {
        let t = Tune {
            lo: 9,
            hi: 2,
            show: 99,
            light: true,
            glow_miss: 99,
            glow_wait: 99,
            replay: 99,
        }
        .clamped(4);
        assert_eq!((t.lo, t.hi), (3, 3));
        assert!(t.show <= 9 && t.glow_wait <= 30);
    }
}
