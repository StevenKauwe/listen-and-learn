//! The spec's acceptance table (§10), encoded as integration tests over
//! the real `Game` + `TouchGate` pair. Rows needing a real browser
//! (scrolling, PWA, offline) live in the Playwright harness instead.

use doremi_core::{Config, Decision, Effect, Event, Game, Phase, TouchGate};

/// Drives one button through the gate exactly as the UI would.
fn tap(g: &mut TouchGate, game: &mut Game, button: u8, t: u32) -> Vec<Effect> {
    let d = g.down(1, button, 0.0, 0.0, t, game.accepts_guess());
    match d {
        Decision::Track => {
            let verdict = g.up(1, t + 120);
            match verdict {
                Decision::Guess(b) => game.handle(Event::GuessSelected { button: b }),
                _ => vec![],
            }
        }
        _ => vec![],
    }
}

#[test]
fn row_touch_before_mystery_plays_registers_nothing() {
    let mut game = Game::new(Config::default());
    let mut gate = TouchGate::new();
    game.handle(Event::StartPressed); // PlayingMystery
    let fx = tap(&mut gate, &mut game, 0, 100);
    assert!(fx.is_empty());
    assert_eq!(game.phase, Phase::PlayingMystery);
    // ...and it was not queued for when guessing opens:
    game.handle(Event::MysteryFinished);
    assert_eq!(game.phase, Phase::AwaitingGuess);
    assert_eq!(game.rounds, 0);
}

#[test]
fn row_finger_held_through_playback_completion_scores_nothing() {
    let mut game = Game::new(Config::default());
    let mut gate = TouchGate::new();
    game.handle(Event::StartPressed);
    // Finger down DURING playback, held past MysteryFinished, then up:
    assert_eq!(
        gate.down(9, 1, 0.0, 0.0, 50, game.accepts_guess()),
        Decision::Ignore
    );
    game.handle(Event::MysteryFinished);
    assert_eq!(gate.up(9, 2000), Decision::Ignore);
    assert_eq!(game.rounds, 0);
}

#[test]
fn row_drag_twenty_px_registers_one_guess() {
    let mut game = Game::new(Config::default());
    let mut gate = TouchGate::new();
    game.handle(Event::StartPressed);
    game.handle(Event::MysteryFinished);
    gate.down(1, 0, 100.0, 100.0, 0, game.accepts_guess());
    gate.r#move(1, 115.0, 108.0);
    let out = match gate.up(1, 300) {
        Decision::Guess(b) => game.handle(Event::GuessSelected { button: b }),
        d => panic!("expected guess, got {d:?}"),
    };
    assert!(matches!(out[0], Effect::PlayGuess { .. }));
}

#[test]
fn row_two_fingers_at_most_one_guess() {
    let mut game = Game::new(Config::default());
    let mut gate = TouchGate::new();
    game.handle(Event::StartPressed);
    let b = game.mystery_button.unwrap();
    game.handle(Event::MysteryFinished);
    gate.down(1, b, 0.0, 0.0, 0, game.accepts_guess());
    assert_eq!(gate.down(2, b, 30.0, 30.0, 5, true), Decision::Ignore);
    gate.up(2, 100);
    gate.up(1, 110);
    // Exactly one scoring path exists per phase transition:
    let fx = game.handle(Event::GuessSelected { button: b });
    assert!(!fx.is_empty());
    assert!(game.handle(Event::GuessSelected { button: b }).is_empty());
}

#[test]
fn row_rapid_ten_taps_no_duplicates_no_broken_transitions() {
    let mut game = Game::new(Config::default());
    let mut gate = TouchGate::new();
    game.handle(Event::StartPressed);
    let b = game.mystery_button.unwrap();
    game.handle(Event::MysteryFinished);
    for i in 0..10 {
        tap(&mut gate, &mut game, b, i * 50);
    }
    // First tap scored; the other nine were phase-gated away:
    assert_eq!(game.phase, Phase::PlayingGuess { correct: true });
    assert_eq!(game.rounds, 0); // rounds increments only on GuessFinished
    for _ in 0..3 {
        game.handle(Event::GuessFinished); // duplicates ignored
    }
    assert_eq!(game.rounds, 1);
}

#[test]
fn row_correct_then_feedback_then_next_round() {
    let mut game = Game::new(Config::default());
    let mut gate = TouchGate::new();
    game.handle(Event::StartPressed);
    let b = game.mystery_button.unwrap();
    game.handle(Event::MysteryFinished);
    tap(&mut gate, &mut game, b, 0);
    game.handle(Event::GuessFinished);
    assert_eq!(game.phase, Phase::CorrectFeedback);
    game.handle(Event::FeedbackDone);
    assert_eq!(game.phase, Phase::Transitioning);
    game.handle(Event::NextRound);
    assert_eq!(game.phase, Phase::PlayingMystery);
}

#[test]
fn row_incorrect_gentle_same_note_stays() {
    let mut game = Game::new(Config::default());
    game.handle(Event::StartPressed);
    let b = game.mystery_button.unwrap();
    let other = (b + 1) % game.button_count();
    game.handle(Event::MysteryFinished);
    game.handle(Event::GuessSelected { button: other });
    game.handle(Event::GuessFinished);
    assert_eq!(game.phase, Phase::IncorrectFeedback);
    game.handle(Event::FeedbackDone);
    assert_eq!(game.phase, Phase::AwaitingGuess);
    assert_eq!(game.mystery_button, Some(b));
}

#[test]
fn row_backgrounded_mid_note_recovers() {
    let mut game = Game::new(Config::default());
    game.handle(Event::StartPressed);
    game.handle(Event::Interrupted);
    assert_eq!(game.phase, Phase::Ready);
    game.handle(Event::StartPressed);
    assert_eq!(game.phase, Phase::PlayingMystery);
    // And guessing can never be open without a finished mystery note:
    assert!(!game.accepts_guess());
}

#[test]
fn settings_change_restarts_cleanly() {
    let mut game = Game::new(Config::default());
    game.handle(Event::StartPressed);
    game.handle(Event::MysteryFinished);
    game.handle(Event::RestartRequested);
    assert_eq!(game.phase, Phase::Ready);
    assert!(!game.accepts_guess());
}
