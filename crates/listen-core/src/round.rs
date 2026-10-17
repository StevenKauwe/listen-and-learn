//! The round machine shared by every question game (spec §2.2).
//!
//! Only `Awaiting` accepts an answer, and it is only entered once the
//! question has fully played. Timing comes from outside: the machine asks
//! for a wait with a sequence number and ignores any timer that is not the
//! one it is waiting for, so stale timers can never double-advance.

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
    Right {
        target: u8,
        name: Option<&'static str>,
    },
    Wrong {
        target: u8,
    },
    /// Echo progress: `done` of the pattern copied.
    Progress {
        done: usize,
        total: usize,
    },
    /// Assist glow on this target, or clear it.
    Hint(Option<u8>),
    Speak(&'static str),
}

/// Adult settings a round needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundCfg {
    pub knobs: Knobs,
    pub auto_next: bool,
    pub cheer: bool,
    pub speak: bool,
    pub assist: bool,
    /// Wrong answers before the question replays itself (0 = never).
    pub replay_after: u32,
}

/// Minimum celebration time before the next round starts.
pub const NEXT_AFTER_S: f32 = 1.5;
/// Misses before the assist glow appears (assist mode only).
pub const ASSIST_AFTER: u32 = 3;
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
    rng: Rng,
    seq: u32,
    pending: Option<u32>,
    /// Same-answer run length, so a two-choice game never gives the same
    /// answer more than three times running.
    run: (Option<u8>, u32),
}

impl Round {
    pub fn new(game: &'static dyn Game, cfg: RoundCfg, seed: u32) -> Self {
        let mut rng = Rng::new(seed);
        let q = game.question(cfg.knobs, &mut rng, None);
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
            rng,
            seq: 0,
            pending: None,
            run: (None, 0),
        };
        r.note_run();
        r
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
                vec![Effect::StopAudio, self.progress()]
            }
        }
    }

    fn invalidate(&mut self) {
        self.seq = self.seq.wrapping_add(1);
        self.pending = None;
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

    fn ask(&mut self) -> Vec<Effect> {
        self.phase = Phase::Question;
        self.retry = false;
        self.step = 0;
        let dur = self.q.sound.dur;
        vec![
            self.progress(),
            Effect::Play(self.q.sound.clone()),
            self.wait(dur, true),
        ]
    }

    fn answer(&mut self, t: u8) -> Vec<Effect> {
        self.guesses += 1;
        let k = self.cfg.knobs;
        let sound = self.game.answer_sound(k, t);
        if self.q.answer.get(self.step) == Some(&t) {
            self.step += 1;
            if self.step < self.q.answer.len() {
                return vec![Effect::Play(sound), self.progress()];
            }
            self.phase = Phase::Right;
            self.rights += 1;
            let dur = sound.dur;
            let sound = if self.cfg.cheer {
                sound.with_chime(dur.min(0.6))
            } else {
                sound
            };
            let mut fx = vec![
                Effect::Hint(None),
                Effect::Play(sound),
                Effect::Right {
                    target: t,
                    name: self.q.name,
                },
                self.progress(),
            ];
            if let (true, Some(say)) = (self.cfg.speak, self.q.say) {
                fx.push(Effect::Speak(say));
            }
            fx.push(self.wait(dur.max(NEXT_AFTER_S), false));
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
            let hint = (self.cfg.assist && self.misses >= ASSIST_AFTER).then(|| self.q.answer[0]);
            vec![
                Effect::Play(sound),
                Effect::Wrong { target: t },
                self.progress(),
                Effect::Hint(hint),
                self.wait(hold, false),
            ]
        }
    }

    fn timer(&mut self) -> Vec<Effect> {
        match self.phase {
            Phase::Question => {
                self.phase = Phase::Awaiting;
                vec![]
            }
            Phase::Right => {
                self.next_question();
                if self.cfg.auto_next {
                    self.ask()
                } else {
                    self.phase = Phase::Ready;
                    vec![]
                }
            }
            Phase::Wrong => {
                let echo = self.q.answer.len() > 1;
                let n = self.cfg.replay_after;
                if echo || (n > 0 && self.misses.is_multiple_of(n)) {
                    self.ask()
                } else {
                    self.phase = Phase::Awaiting;
                    vec![]
                }
            }
            Phase::Ready | Phase::Awaiting => vec![],
        }
    }

    fn next_question(&mut self) {
        self.misses = 0;
        self.step = 0;
        for _ in 0..32 {
            let q = self
                .game
                .question(self.cfg.knobs, &mut self.rng, Some(&self.q));
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
