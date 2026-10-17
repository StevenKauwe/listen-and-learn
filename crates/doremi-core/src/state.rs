//! The game's finite-state machine (spec §2).
//!
//! Seven phases, one rule above all: **only `AwaitingGuess` accepts a
//! guess**, and only from a touch that *began* in `AwaitingGuess` (a finger
//! already resting on a button when guessing enables must lift and touch
//! again). Invalid events are ignored entirely — never queued.

use crate::config::Config;

/// The phases. Exactly one is active at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Big central "Listen!" button; note buttons visible but inactive.
    Ready,
    /// The mystery note is sounding. Guesses impossible.
    PlayingMystery,
    /// The note finished; guessing open.
    AwaitingGuess,
    /// The child's chosen note is sounding; evaluation is decided but
    /// feedback waits until it finishes playing.
    PlayingGuess { correct: bool },
    /// Celebration; next round auto-follows after the interval.
    CorrectFeedback,
    /// Gentle wiggle; same mystery note stays.
    IncorrectFeedback,
    /// Between rounds; a timer owns the return to PlayingMystery.
    Transitioning,
}

/// Events fed in by the UI layer. Timing events (audio finished, timers)
/// arrive from outside so tests never sleep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The big button was pressed (valid in Ready). Also used to retry
    /// after audio failure.
    StartPressed,
    /// The mystery note actually started sounding (audio unlocked).
    MysteryStarted,
    /// The mystery note finished sounding.
    MysteryFinished,
    /// The child selected button `idx` (already gated by the TouchGate
    /// AND by the phase — see `Game::accepts_guess`).
    GuessSelected { button: u8 },
    /// The guessed note finished sounding.
    GuessFinished,
    /// The feedback interval elapsed.
    FeedbackDone,
    /// The between-rounds timer fired.
    NextRound,
    /// "Play it again" (valid in AwaitingGuess).
    ReplayRequested,
    /// Audio failed to start (autoplay block, interruption). Never
    /// advance to AwaitingGuess on failure.
    AudioFailed,
    /// The app was backgrounded/suspended mid-anything. Recover to a
    /// clean, safe state with no penalty.
    Interrupted,
    /// Adult changed settings mid-game: restart cleanly.
    RestartRequested,
}

/// What the UI should do as a result of an event. Effects are commands,
/// never decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Unlock/init audio, then sound the mystery note (scale index).
    PlayMystery { scale_idx: usize },
    /// Sound the guessed button's note (scale index).
    PlayGuess { scale_idx: usize },
    /// Show celebration (correct=true) or the gentle wiggle.
    ShowFeedback { correct: bool, scale_idx: usize },
    /// Arm a timer for the given milliseconds, then send `NextRound`.
    ScheduleNextRound { ms: u32 },
    /// Arm a timer for the feedback interval, then send `FeedbackDone`.
    ScheduleFeedbackDone { ms: u32 },
    /// Speak the note's name (if enabled in config).
    Speak { name: &'static str },
    /// Re-enable/refresh the answer buttons for guessing.
    ButtonsEnabled(bool),
    /// Audio is broken; show an obvious retry control.
    ShowRetry,
    /// Play the celebration flourish (if enabled).
    CelebrationSound,
}

/// Feedback interval (spec: next round after ~1.5 s on success).
pub const FEEDBACK_MS: u32 = 1500;
/// Wiggle time on a miss before buttons re-open.
pub const TRY_AGAIN_MS: u32 = 900;

/// The game.
pub struct Game {
    pub phase: Phase,
    pub config: Config,
    /// The current round's mystery note, as a *button* index in the
    /// active set. `None` until the first round starts.
    pub mystery_button: Option<u8>,
    /// Scale index of the mystery note (what audio plays).
    pub mystery_scale_idx: usize,
    /// Wrong guesses against the current mystery note.
    wrong_this_round: u32,
    /// Consecutive correct answers (for optional stars).
    pub consecutive_correct: u32,
    /// Completed rounds.
    pub rounds: u32,
    /// Simple xorshift so note picks are deterministic in tests.
    rng: u32,
}

impl Game {
    pub fn new(config: Config) -> Self {
        let rng = config_note_seed();
        Self {
            phase: Phase::Ready,
            config: config.sanitized(),
            mystery_button: None,
            mystery_scale_idx: 0,
            wrong_this_round: 0,
            consecutive_correct: 0,
            rounds: 0,
            rng,
        }
    }

    /// Which difficulty's button count is active.
    pub fn button_count(&self) -> u8 {
        self.config.difficulty.scale_indices().len() as u8
    }

    /// Scale index of an active button (for coloring/labels/audio).
    pub fn scale_idx_of_button(&self, button: u8) -> usize {
        crate::notes::scale_index_for(self.config.difficulty, button)
    }

    /// Whether a *fresh* touch on an answer button would count right now.
    /// The TouchGate enforces "fresh"; this enforces "now".
    pub fn accepts_guess(&self) -> bool {
        self.phase == Phase::AwaitingGuess
    }

    /// Pick the next mystery note: uniform over the active set, optionally
    /// avoiding an immediate repeat when there is more than one note.
    fn pick_mystery(&mut self) -> u8 {
        let n = self.button_count();
        if n <= 1 {
            return 0;
        }
        let mut choice = (self.next_random() % n as u32) as u8;
        if Some(choice) == self.mystery_button {
            choice = (choice + 1) % n; // never the same note twice running
        }
        choice
    }

    fn next_random(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }

    /// Feed one event; receive the effects to perform. Events that do not
    /// apply to the current phase are dropped on the floor — this is what
    /// makes double activations and overlapping transitions impossible.
    pub fn handle(&mut self, event: Event) -> Vec<Effect> {
        use Effect::*;
        use Event::*;
        use Phase::*;

        match (self.phase, event) {
            // Cross-phase events first: they win over per-phase wildcards.
            (_, RestartRequested) => {
                self.phase = Ready;
                self.wrong_this_round = 0;
                vec![ButtonsEnabled(false)]
            }
            (_, Interrupted) => {
                self.phase = Ready;
                vec![ButtonsEnabled(false)]
            }
            // --- Ready: the big button -------------------------------
            (Ready, StartPressed) => {
                let b = self.pick_mystery();
                self.mystery_button = Some(b);
                self.mystery_scale_idx = self.scale_idx_of_button(b);
                self.wrong_this_round = 0;
                self.phase = PlayingMystery;
                vec![PlayMystery { scale_idx: self.mystery_scale_idx }]
            }
            // --- The mystery note is sounding ------------------------
            (PlayingMystery, MysteryFinished) => {
                self.phase = AwaitingGuess;
                vec![ButtonsEnabled(true)]
            }
            (PlayingMystery, AudioFailed) => {
                self.phase = Ready;
                vec![ShowRetry, ButtonsEnabled(false)]
            }
            (PlayingMystery, _) => vec![], // incl. premature GuessSelected

            // --- Guessing open ---------------------------------------
            (AwaitingGuess, GuessSelected { button }) => {
                // The gate already checked freshness; we check phase.
                let idx = self.scale_idx_of_button(button);
                let correct = Some(button) == self.mystery_button;
                self.phase = PlayingGuess { correct };
                vec![PlayGuess { scale_idx: idx }]
            }
            (AwaitingGuess, ReplayRequested) => {
                self.phase = PlayingMystery; // same note, again
                vec![PlayMystery { scale_idx: self.mystery_scale_idx }]
            }
            (AwaitingGuess, _) => vec![],

            // --- The guessed note is sounding ------------------------
            (PlayingGuess { correct }, GuessFinished) => {
                if correct {
                    self.consecutive_correct += 1;
                    self.rounds += 1;
                    self.phase = CorrectFeedback;
                    let mut fx = vec![
                        ShowFeedback { correct: true, scale_idx: self.mystery_scale_idx },
                        ScheduleFeedbackDone { ms: FEEDBACK_MS },
                    ];
                    if self.config.spoken_names {
                        let name = crate::notes::NOTE_TABLE[self.mystery_scale_idx].name;
                        fx.push(Speak { name });
                    }
                    if self.config.celebration_sounds {
                        fx.push(CelebrationSound);
                    }
                    fx
                } else {
                    self.wrong_this_round += 1;
                    self.consecutive_correct = 0;
                    self.phase = IncorrectFeedback;
                    vec![
                        ShowFeedback { correct: false, scale_idx: idx_of(self) },
                        ScheduleFeedbackDone { ms: TRY_AGAIN_MS },
                    ]
                }
            }
            (PlayingGuess { .. }, AudioFailed) => {
                // Evaluation was already decided; land the feedback now.
                self.phase = Ready;
                vec![ShowRetry, ButtonsEnabled(false)]
            }
            (PlayingGuess { .. }, _) => vec![],

            // --- Feedback --------------------------------------------
            (CorrectFeedback, FeedbackDone) => {
                if self.config.auto_next_round {
                    self.phase = Transitioning;
                    vec![ScheduleNextRound { ms: 250 }, ButtonsEnabled(false)]
                } else {
                    // Adult turned auto-next off: wait on the big button.
                    self.phase = Ready;
                    vec![ButtonsEnabled(false)]
                }
            }
            (IncorrectFeedback, FeedbackDone) => {
                // After repeated misses: replay the note for the child
                // (and assist mode highlights — the UI reads config).
                if self.config.auto_replay_after > 0
                    && self.wrong_this_round >= self.config.auto_replay_after
                {
                    self.phase = PlayingMystery; // same mystery note
                    vec![PlayMystery { scale_idx: self.mystery_scale_idx }]
                } else {
                    self.phase = AwaitingGuess; // same mystery note
                    vec![ButtonsEnabled(true)]
                }
            }
            // --- Between rounds --------------------------------------
            (Transitioning, NextRound) => {
                let b = self.pick_mystery();
                self.mystery_button = Some(b);
                self.mystery_scale_idx = self.scale_idx_of_button(b);
                self.wrong_this_round = 0;
                self.phase = PlayingMystery;
                vec![PlayMystery { scale_idx: self.mystery_scale_idx }]
            }
            (Transitioning, _) => vec![],

            // --- audio-start confirmations (ignored where N/A) --------
            (_, MysteryStarted) => vec![],
            (_, AudioFailed) => vec![],
            (_, _) => vec![],
        }
    }
}

fn idx_of(_g: &Game) -> usize {
    // The incorrect-feedback wiggle doesn't need a note; keep the slot.
    usize::MAX
}

fn config_note_seed() -> u32 {
    // A fixed seed keeps tests deterministic; the UI may reseed from time
    // via `Game { rng, .. }` construction or a future `reseed` method.
    0x9E3779B9
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game::new(Config::default())
    }

    #[test]
    fn ready_does_not_accept_guesses() {
        let mut g = game();
        assert!(g.handle(Event::GuessSelected { button: 0 }).is_empty());
        assert_eq!(g.phase, Phase::Ready);
    }

    #[test]
    fn full_correct_round_then_next() {
        let mut g = game();
        g.handle(Event::StartPressed);
        let b = g.mystery_button.unwrap();
        g.handle(Event::MysteryFinished);
        let fx = g.handle(Event::GuessSelected { button: b });
        assert!(matches!(fx[0], Effect::PlayGuess { .. }));
        let fx = g.handle(Event::GuessFinished);
        assert!(fx.iter().any(|e| matches!(e, Effect::ShowFeedback { correct: true, .. })));
        g.handle(Event::FeedbackDone);
        assert_eq!(g.phase, Phase::Transitioning);
        g.handle(Event::NextRound);
        assert_eq!(g.phase, Phase::PlayingMystery);
    }

    #[test]
    fn wrong_guess_keeps_the_same_note() {
        let mut g = game();
        g.handle(Event::StartPressed);
        let b = g.mystery_button.unwrap();
        let other = (b + 1) % g.button_count();
        g.handle(Event::MysteryFinished);
        g.handle(Event::GuessSelected { button: other });
        g.handle(Event::GuessFinished);
        assert_eq!(g.phase, Phase::IncorrectFeedback);
        g.handle(Event::FeedbackDone);
        assert_eq!(g.phase, Phase::AwaitingGuess);
        assert_eq!(g.mystery_button, Some(b)); // unchanged
    }

    #[test]
    fn premature_guess_is_dropped_never_queued() {
        let mut g = game();
        g.handle(Event::StartPressed); // PlayingMystery
        assert!(g.handle(Event::GuessSelected { button: 0 }).is_empty());
        g.handle(Event::MysteryFinished);
        // The dropped guess must NOT have been queued:
        assert_eq!(g.phase, Phase::AwaitingGuess);
        assert_eq!(g.wrong_this_round, 0);
        assert_eq!(g.rounds, 0);
    }

    #[test]
    fn double_scoring_is_impossible() {
        let mut g = game();
        g.handle(Event::StartPressed);
        let b = g.mystery_button.unwrap();
        g.handle(Event::MysteryFinished);
        g.handle(Event::GuessSelected { button: b });
        // A second "guess" while the first plays:
        assert!(g.handle(Event::GuessSelected { button: b }).is_empty());
        g.handle(Event::GuessFinished);
        g.handle(Event::GuessFinished); // duplicate finish: ignored
        assert_eq!(g.rounds, 1);
    }

    #[test]
    fn audio_failure_never_opens_guessing() {
        let mut g = game();
        g.handle(Event::StartPressed);
        let fx = g.handle(Event::AudioFailed);
        assert!(fx.iter().any(|e| matches!(e, Effect::ShowRetry)));
        assert_eq!(g.phase, Phase::Ready);
        assert!(!g.accepts_guess());
    }

    #[test]
    fn interruption_recovers_cleanly_anywhere() {
        for mut g in [game(), {
            let mut g = game();
            g.handle(Event::StartPressed);
            g
        }] {
            g.handle(Event::Interrupted);
            assert_eq!(g.phase, Phase::Ready);
            // And the game can start again:
            g.handle(Event::StartPressed);
            assert_eq!(g.phase, Phase::PlayingMystery);
        }
    }

    #[test]
    fn two_misses_trigger_auto_replay() {
        let mut g = game();
        g.handle(Event::StartPressed);
        let b = g.mystery_button.unwrap();
        let other = (b + 1) % g.button_count();
        for _ in 0..2 {
            g.handle(Event::MysteryFinished);
            g.handle(Event::GuessSelected { button: other });
            g.handle(Event::GuessFinished);
            g.handle(Event::FeedbackDone);
        }
        // After the 2nd miss (auto_replay_after = 2): replay, same note.
        assert_eq!(g.phase, Phase::PlayingMystery);
        assert_eq!(g.mystery_button, Some(b));
    }

    #[test]
    fn never_same_note_twice_in_a_row() {
        let mut g = game();
        let mut last = None;
        for _ in 0..12 {
            g.handle(Event::StartPressed);
            let b = g.mystery_button.unwrap();
            if let Some(prev) = last {
                assert_ne!(prev, b);
            }
            last = Some(b);
            g.handle(Event::MysteryFinished);
            g.handle(Event::GuessSelected { button: b });
            g.handle(Event::GuessFinished);
            g.handle(Event::FeedbackDone);
            g.handle(Event::NextRound);
        }
    }
}
