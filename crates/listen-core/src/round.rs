//! The round machine shared by every question game (spec §2.2).
//!
//! Only `Awaiting` accepts an answer, and it is only entered once the
//! question has fully played. Timing comes from outside: the machine asks
//! for a wait with a sequence number and ignores any timer that is not the
//! one it is waiting for, so stale timers can never double-advance.
//!
//! Help (calib.rs) is decided here too: the first right answers are shown
//! (the answer lights, glows, and the other choices are dimmed and only
//! make their sound), later ones may be lit while the question plays, and
//! a glow arrives after misses or after a wait. The game's step adapts
//! within the tune's band.

use crate::calib::{Help, Tune, RESHOW_AFTER, STEP_DOWN_MISSES, STEP_UP_AFTER};
use crate::games::{Game, Knobs, Question};
use crate::music::Sound;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Listen button waiting; answers visible but closed.
    Ready,
    /// The question is sounding. Answers closed.
    Question,
    /// The question finished; answers open.
    Awaiting,
    /// A correct answer is sounding and celebrating.
    Right,
    /// A wrong answer is sounding; the same question stays.
    Wrong,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Listen pressed (also: retry after audio failure, replay).
    Listen,
    /// A gated, accepted touch on answer target `t`.
    Answer(u8),
    /// The wait with this sequence number elapsed.
    Timer(u32),
    /// The delayed-glow wait with this sequence number elapsed.
    HintTimer(u32),
    /// Audio could not start or stopped. Never open answers after this.
    AudioFailed,
    /// Page hidden, interrupted or suspended.
    Interrupted,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Effect {
    Play(Sound),
    StopAudio,
    /// Send `Timer(seq)` after `secs`. `audio_clock`: measure on the audio
    /// clock (the question must really have played).
    Wait {
        seq: u32,
        secs: f32,
        audio_clock: bool,
    },
    /// Send `HintTimer(seq)` after `secs` (wall clock).
    HintAfter {
        seq: u32,
        secs: f32,
    },
    Right {
        target: u8,
        name: Option<&'static str>,
    },
    Wrong {
        target: u8,
    },
    /// A touch on a dimmed choice while the answer is shown: it only makes
    /// its sound (exploring is fine, nothing is wrong).
    Poke {
        target: u8,
    },
    /// Echo progress: `done` of the pattern copied.
    Progress {
        done: usize,
        total: usize,
    },
    /// Glow on this target, or clear it.
    Hint(Option<u8>),
    /// Dim every choice but this one, or undim all.
    Focus(Option<u8>),
    /// The game stepped to another rung of its ladder.
    Step(u8),
    /// A set is finished: celebrate and rest until Listen.
    SetDone,
}

/// Adult settings a round needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundCfg {
    pub knobs: Knobs,
    pub tune: Tune,
    pub auto_next: bool,
    pub cheer: bool,
    /// Sing the note name after a right answer (Do Re Mi).
    pub sing: bool,
    /// Right answers in a set before a rest (0 = no rest).
    pub set_len: u8,
}

/// Minimum celebration time before the next round starts.
pub const NEXT_AFTER_S: f32 = 1.5;
/// Pause after an Echo slip before the pattern plays again.
pub const ECHO_REPLAY_S: f32 = 0.9;

pub struct Round {
    pub game: &'static dyn Game,
    pub cfg: RoundCfg,
    pub phase: Phase,
    pub q: Question,
    /// The last audio attempt failed; show retry.
    pub retry: bool,
    /// Echo: how much of the pattern is copied.
    pub step: usize,
    pub misses: u32,
    pub guesses: u32,
    pub rights: u32,
    pub wrongs: u32,
    /// Current rung of the game's ladder.
    pub level: u8,
    /// Right answers still to be shown.
    pub show_left: u8,
    /// Right answers in the current set.
    pub in_set: u32,
    rng: Rng,
    seq: u32,
    pending: Option<u32>,
    hint_seq: u32,
    hint_pending: Option<u32>,
    streak: u32,
    struggles: u32,
    /// Same-answer run length, so a single-answer game never gives the
    /// same answer more than three times running.
    run: (Option<u8>, u32),
}

impl Round {
    pub fn new(game: &'static dyn Game, cfg: RoundCfg, seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let tune = cfg.tune.clamped(game.steps().len());
        let cfg = RoundCfg { tune, ..cfg };
        let level = tune.lo;
        let q = game.question(Knobs { level, ..cfg.knobs }, &mut rng, None);
        let mut r = Round {
            game,
            cfg,
            phase: Phase::Ready,
            q,
            retry: false,
            step: 0,
            misses: 0,
            guesses: 0,
            rights: 0,
            wrongs: 0,
            level,
            show_left: tune.show,
            in_set: 0,
            rng,
            seq: 0,
            pending: None,
            hint_seq: 0,
            hint_pending: None,
            streak: 0,
            struggles: 0,
            run: (None, 0),
        };
        r.note_run();
        r
    }

    pub fn knobs(&self) -> Knobs {
        Knobs {
            level: self.level,
            ..self.cfg.knobs
        }
    }

    /// How much this question shows its answer.
    pub fn help(&self) -> Help {
        if self.show_left > 0 {
            Help::Show
        } else if self.cfg.tune.light {
            Help::Light
        } else {
            Help::None
        }
    }

    /// The targets on screen (fixed for the whole visit).
    pub fn targets(&self) -> Vec<u8> {
        self.game.targets(self.cfg.tune.lo)
    }

    pub fn accepts_answer(&self) -> bool {
        self.phase == Phase::Awaiting
    }

    pub fn accepts_listen(&self) -> bool {
        matches!(self.phase, Phase::Ready | Phase::Awaiting)
    }

    pub fn handle(&mut self, ev: Event) -> Vec<Effect> {
        match ev {
            Event::Listen if self.accepts_listen() => self.ask(),
            Event::Listen => vec![],
            Event::Answer(t) if self.accepts_answer() => self.answer(t),
            Event::Answer(_) => vec![],
            Event::Timer(s) if self.pending == Some(s) => {
                self.pending = None;
                self.timer()
            }
            Event::Timer(_) => vec![],
            Event::HintTimer(s) if self.hint_pending == Some(s) => {
                self.hint_pending = None;
                if self.phase == Phase::Awaiting {
                    vec![Effect::Hint(Some(self.expected()))]
                } else {
                    vec![]
                }
            }
            Event::HintTimer(_) => vec![],
            Event::AudioFailed => {
                self.invalidate();
                self.phase = Phase::Ready;
                self.retry = true;
                self.step = 0;
                vec![Effect::StopAudio, self.progress()]
            }
            Event::Interrupted => {
                self.invalidate();
                if self.phase == Phase::Right {
                    self.next_question();
                }
                self.phase = Phase::Ready;
                self.step = 0;
                vec![Effect::StopAudio, self.progress(), Effect::Hint(None)]
            }
        }
    }

    fn expected(&self) -> u8 {
        self.q.answer[self.step.min(self.q.answer.len() - 1)]
    }

    fn invalidate(&mut self) {
        self.seq = self.seq.wrapping_add(1);
        self.pending = None;
        self.hint_seq = self.hint_seq.wrapping_add(1);
        self.hint_pending = None;
    }

    fn wait(&mut self, secs: f32, audio_clock: bool) -> Effect {
        self.invalidate();
        self.pending = Some(self.seq);
        Effect::Wait {
            seq: self.seq,
            secs,
            audio_clock,
        }
    }

    fn progress(&self) -> Effect {
        Effect::Progress {
            done: self.step,
            total: self.q.answer.len(),
        }
    }

    /// The question as it sounds at the current help: lights only when
    /// the answer is shown or lit.
    fn question_sound(&self) -> Sound {
        let mut s = self.q.sound.clone();
        if self.help() == Help::None {
            for t in s.tones.iter_mut() {
                t.light = None;
            }
        }
        s
    }

    fn ask(&mut self) -> Vec<Effect> {
        self.phase = Phase::Question;
        self.retry = false;
        self.step = 0;
        let dur = self.q.sound.dur;
        let focus = (self.help() == Help::Show).then(|| self.q.answer[0]);
        let sound = self.question_sound();
        vec![
            self.progress(),
            Effect::Hint(None),
            Effect::Focus(focus),
            Effect::Play(sound),
            self.wait(dur, true),
        ]
    }

    /// Answers open: glow at once when shown, else arm the delayed glow.
    fn open(&mut self) -> Vec<Effect> {
        self.phase = Phase::Awaiting;
        if self.help() == Help::Show {
            return vec![Effect::Hint(Some(self.expected()))];
        }
        let secs = self.cfg.tune.glow_wait;
        if secs == 0 {
            return vec![];
        }
        self.hint_seq = self.hint_seq.wrapping_add(1);
        self.hint_pending = Some(self.hint_seq);
        vec![Effect::HintAfter {
            seq: self.hint_seq,
            secs: secs as f32,
        }]
    }

    fn answer(&mut self, t: u8) -> Vec<Effect> {
        let k = self.knobs();
        let sound = self.game.answer_sound(k, t);
        let expected = self.expected();
        if t != expected && self.help() == Help::Show {
            // Errorless: a dimmed choice only makes its sound.
            return vec![Effect::Play(sound), Effect::Poke { target: t }];
        }
        self.guesses += 1;
        if t == expected {
            self.step += 1;
            if self.step < self.q.answer.len() {
                let mut fx = vec![Effect::Play(sound), self.progress()];
                if self.help() == Help::Show {
                    fx.push(Effect::Hint(Some(self.expected())));
                }
                return fx;
            }
            self.phase = Phase::Right;
            self.rights += 1;
            self.in_set += 1;
            let dur = sound.dur;
            let mut sound = sound;
            if let (true, Some((deg, midi))) = (self.cfg.sing, self.q.sing) {
                sound = sound.with_name(deg, midi, crate::games::notes::SING_AT);
            }
            let total = sound.dur;
            if self.cfg.cheer {
                sound = sound.with_chime(total.min(dur.max(0.6) + 0.5).min(total));
            }
            let mut fx = vec![
                Effect::Hint(None),
                Effect::Focus(None),
                Effect::Play(sound),
                Effect::Right {
                    target: t,
                    name: self.q.name,
                },
                self.progress(),
            ];
            fx.extend(self.adapt_after_right());
            fx.push(self.wait(total.max(NEXT_AFTER_S), false));
            fx
        } else {
            self.phase = Phase::Wrong;
            self.wrongs += 1;
            self.misses += 1;
            self.step = 0;
            let echo = self.q.answer.len() > 1;
            let hold = if echo {
                sound.dur + ECHO_REPLAY_S
            } else {
                sound.dur
            };
            let n = self.cfg.tune.glow_miss as u32;
            let hint = (n > 0 && self.misses >= n).then(|| self.q.answer[0]);
            vec![
                Effect::Play(sound),
                Effect::Wrong { target: t },
                self.progress(),
                Effect::Hint(hint),
                self.wait(hold, false),
            ]
        }
    }

    /// Shown answers count down; first-try successes step harder.
    fn adapt_after_right(&mut self) -> Vec<Effect> {
        let mut fx = vec![];
        if self.show_left > 0 {
            self.show_left -= 1;
            return fx;
        }
        if self.misses == 0 {
            self.streak += 1;
            self.struggles = 0;
            if self.streak >= STEP_UP_AFTER && self.level < self.cfg.tune.hi {
                if self.same_targets(self.level + 1) {
                    self.level += 1;
                    fx.push(Effect::Step(self.level));
                }
                self.streak = 0;
            }
        } else {
            self.streak = 0;
        }
        fx
    }

    fn same_targets(&self, level: u8) -> bool {
        self.game.targets(level) == self.game.targets(self.level)
    }

    /// A question that took several misses steps easier; a run of them
    /// brings back one shown answer.
    fn adapt_after_struggle(&mut self) {
        if self.misses < STEP_DOWN_MISSES {
            return;
        }
        self.streak = 0;
        self.struggles += 1;
        if self.level > self.cfg.tune.lo && self.same_targets(self.level - 1) {
            self.level -= 1;
        }
        if self.struggles >= RESHOW_AFTER && self.cfg.tune.show > 0 {
            self.show_left = 1;
            self.struggles = 0;
        }
    }

    fn timer(&mut self) -> Vec<Effect> {
        match self.phase {
            Phase::Question => self.open(),
            Phase::Right => {
                self.next_question();
                let n = self.cfg.set_len as u32;
                if n > 0 && self.in_set >= n {
                    self.in_set = 0;
                    self.phase = Phase::Ready;
                    return vec![Effect::SetDone];
                }
                if self.cfg.auto_next {
                    self.ask()
                } else {
                    self.phase = Phase::Ready;
                    vec![]
                }
            }
            Phase::Wrong => {
                let echo = self.q.answer.len() > 1;
                let n = self.cfg.tune.replay as u32;
                if echo || (n > 0 && self.misses.is_multiple_of(n)) {
                    // A replay keeps the glow if one was earned.
                    let keep = self.misses >= self.cfg.tune.glow_miss as u32
                        && self.cfg.tune.glow_miss > 0;
                    let mut fx = self.ask();
                    if keep {
                        fx.retain(|e| *e != Effect::Hint(None));
                    }
                    fx
                } else {
                    self.open()
                }
            }
            Phase::Ready | Phase::Awaiting => vec![],
        }
    }

    fn next_question(&mut self) {
        self.adapt_after_struggle();
        self.misses = 0;
        self.step = 0;
        let k = self.knobs();
        for _ in 0..32 {
            let q = self.game.question(k, &mut self.rng, Some(&self.q));
            let single = q.answer.len() == 1;
            let fourth = single && self.run.0 == q.answer.first().copied() && self.run.1 >= 3;
            self.q = q;
            if !fourth {
                break;
            }
        }
        self.note_run();
    }

    fn note_run(&mut self) {
        let a = self.q.answer.first().copied();
        self.run = if self.run.0 == a {
            (a, self.run.1 + 1)
        } else {
            (a, 1)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calib::{preset, Age};
    use crate::games::by_slug;

    fn cfg(slug: &str, age: Age) -> RoundCfg {
        let mut s = crate::Settings::default();
        s.set_age(age);
        s.round_cfg(by_slug(slug).unwrap().0)
    }

    fn seq(fx: &[Effect]) -> u32 {
        fx.iter()
            .find_map(|e| match e {
                Effect::Wait { seq, .. } => Some(*seq),
                _ => None,
            })
            .unwrap()
    }

    fn hear(r: &mut Round) -> Vec<Effect> {
        let fx = r.handle(Event::Listen);
        r.handle(Event::Timer(seq(&fx)))
    }

    fn right(r: &mut Round) {
        let fx = r.handle(Event::Answer(r.q.answer[0]));
        r.handle(Event::Timer(seq(&fx)));
        r.handle(Event::Interrupted);
    }

    #[test]
    fn age_two_first_rounds_are_errorless_and_glow_at_once() {
        let mut r = Round::new(by_slug("notes").unwrap().1, cfg("notes", Age::Two), 1);
        assert_eq!(r.help(), Help::Show);
        let fx = r.handle(Event::Listen);
        assert!(fx.contains(&Effect::Focus(Some(r.q.answer[0]))));
        let lit = fx
            .iter()
            .any(|e| matches!(e, Effect::Play(s) if s.tones[0].light.is_some()));
        assert!(lit, "the bar lights while it plays");
        let fx = r.handle(Event::Timer(seq(&fx)));
        assert_eq!(fx, vec![Effect::Hint(Some(r.q.answer[0]))]);
        let other = *r.targets().iter().find(|&&t| t != r.q.answer[0]).unwrap();
        let fx = r.handle(Event::Answer(other));
        assert!(fx.contains(&Effect::Poke { target: other }));
        assert_eq!((r.phase, r.guesses, r.wrongs), (Phase::Awaiting, 0, 0));
    }

    #[test]
    fn shown_rounds_fade_to_lit_then_misses_bring_them_back() {
        let mut c = cfg("twins", Age::Two);
        c.tune.replay = 0;
        let mut r = Round::new(by_slug("twins").unwrap().1, c, 3);
        for _ in 0..c.tune.show {
            hear(&mut r);
            right(&mut r);
        }
        assert_eq!(r.help(), Help::Light, "age 2 keeps the light");
        for _ in 0..RESHOW_AFTER {
            hear(&mut r);
            for _ in 0..STEP_DOWN_MISSES {
                let wrong = 1 - r.q.answer[0];
                let fx = r.handle(Event::Answer(wrong));
                r.handle(Event::Timer(seq(&fx)));
            }
            right(&mut r);
        }
        assert_eq!(r.help(), Help::Show, "the answer is shown again");
    }

    #[test]
    fn first_try_successes_step_harder_within_the_band_only() {
        let c = cfg("twins", Age::Five);
        let t = preset(Age::Five, "twins");
        let mut r = Round::new(by_slug("twins").unwrap().1, c, 5);
        assert_eq!(r.level, t.lo);
        for _ in 0..40 {
            hear(&mut r);
            right(&mut r);
        }
        assert_eq!(r.level, t.hi);
    }

    #[test]
    fn delayed_glow_arrives_only_after_the_wait_and_never_stale() {
        let c = cfg("birdfish", Age::Three);
        let mut r = Round::new(by_slug("birdfish").unwrap().1, c, 9);
        r.show_left = 0;
        let fx = hear(&mut r);
        let Some(Effect::HintAfter { seq: s, secs }) = fx.first().cloned() else {
            panic!("{fx:?}")
        };
        assert_eq!(secs, c.tune.glow_wait as f32);
        assert_eq!(
            r.handle(Event::HintTimer(s)),
            vec![Effect::Hint(Some(r.q.answer[0]))]
        );
        let fx = hear(&mut r);
        let Some(Effect::HintAfter { seq: s2, .. }) = fx.first().cloned() else {
            panic!()
        };
        r.handle(Event::Answer(r.q.answer[0]));
        assert!(
            r.handle(Event::HintTimer(s2)).is_empty(),
            "answered already"
        );
    }

    #[test]
    fn a_set_ends_with_a_rest_then_listen_starts_the_next() {
        let c = cfg("loud", Age::Two);
        let mut r = Round::new(by_slug("loud").unwrap().1, c, 2);
        let mut done = 0;
        for _ in 0..c.set_len {
            hear(&mut r);
            let fx = r.handle(Event::Answer(r.q.answer[0]));
            let fx = r.handle(Event::Timer(seq(&fx)));
            if fx.contains(&Effect::SetDone) {
                done += 1;
            } else {
                r.handle(Event::Interrupted);
            }
        }
        assert_eq!(done, 1);
        assert_eq!(r.phase, Phase::Ready);
        assert_eq!(r.handle(Event::Listen).len(), 5);
    }

    #[test]
    fn sung_name_follows_a_right_do_re_mi_answer_at_its_pitch() {
        let mut c = cfg("notes", Age::Five);
        c.sing = true;
        let mut r = Round::new(by_slug("notes").unwrap().1, c, 4);
        hear(&mut r);
        let fx = r.handle(Event::Answer(r.q.answer[0]));
        let sound = fx
            .iter()
            .find_map(|e| match e {
                Effect::Play(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap();
        let (deg, midi) = r.q.sing.unwrap();
        let sung = sound
            .tones
            .iter()
            .find(|t| t.voice == crate::music::Voice::Sing(deg % 7))
            .expect("a sung name");
        assert_eq!(sung.midi, midi as f32);
    }
}
