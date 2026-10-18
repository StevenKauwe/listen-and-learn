//! Spec acceptance rows that are about behaviour, for every game.
//! Browser-only rows (no scroll, rotate, real audio) live in tests-browser/.

use listen_core::*;

/// A by-ear configuration (age 4 bands, no help), so wrong answers count.
fn cfg(game: usize) -> RoundCfg {
    let mut s = Settings::default();
    s.set_age(Age::Four);
    let mut c = s.round_cfg(game);
    c.tune.show = 0;
    c.tune.light = false;
    c.tune.glow_wait = 0;
    c.tune.glow_miss = 0;
    c.tune.replay = 2;
    c.tune.hi = c.tune.lo;
    c.set_len = 0;
    c
}

fn wait_seq(fx: &[Effect]) -> Option<u32> {
    fx.iter().find_map(|e| match e {
        Effect::Wait { seq, .. } => Some(*seq),
        _ => None,
    })
}

/// Press Listen and let the question finish.
fn hear(r: &mut Round) {
    let fx = r.handle(Event::Listen);
    assert_eq!(r.phase, Phase::Question);
    r.handle(Event::Timer(wait_seq(&fx).expect("question waits")));
    assert_eq!(r.phase, Phase::Awaiting);
}

fn wrong_target(r: &Round) -> u8 {
    let t = r.q.answer[r.step];
    *r.targets().iter().find(|&&x| x != t).unwrap()
}

#[test]
fn every_game_plays_one_wrong_then_one_right_and_moves_on() {
    for (i, g) in ALL
        .iter()
        .enumerate()
        .filter(|(_, g)| !g.free() && !g.beat())
    {
        let mut r = Round::new(*g, cfg(i), 42);
        hear(&mut r);
        let before = r.q.clone();
        let fx = r.handle(Event::Answer(wrong_target(&r)));
        assert_eq!(r.phase, Phase::Wrong, "{}", g.slug());
        assert!(fx.iter().any(|e| matches!(e, Effect::Wrong { .. })));
        let fx = r.handle(Event::Timer(wait_seq(&fx).unwrap()));
        assert_eq!(r.q, before, "{}: the same question stays", g.slug());
        if r.phase == Phase::Question {
            r.handle(Event::Timer(wait_seq(&fx).unwrap())); // Echo replays
        }
        assert_eq!(r.phase, Phase::Awaiting);
        let answer = r.q.answer.clone();
        let mut fx = vec![];
        for t in answer {
            fx = r.handle(Event::Answer(t));
        }
        assert_eq!(r.phase, Phase::Right, "{}", g.slug());
        assert!(fx.iter().any(|e| matches!(e, Effect::Play(_))));
        assert_eq!((r.rights, r.wrongs), (1, 1));
        r.handle(Event::Timer(wait_seq(&fx).unwrap()));
        assert_eq!(
            r.phase,
            Phase::Question,
            "{}: auto-next asks again",
            g.slug()
        );
    }
}

#[test]
fn no_answer_before_the_question_has_fully_played() {
    let mut r = Round::new(ALL[0], cfg(0), 1);
    assert!(r.handle(Event::Answer(r.q.answer[0])).is_empty()); // before Listen
    r.handle(Event::Listen);
    assert!(r.handle(Event::Answer(r.q.answer[0])).is_empty()); // while sounding
    assert_eq!(r.guesses, 0);
}

#[test]
fn ten_rapid_taps_score_once() {
    let mut r = Round::new(ALL[0], cfg(0), 1);
    hear(&mut r);
    let a = r.q.answer[0];
    for _ in 0..10 {
        r.handle(Event::Answer(a));
        r.handle(Event::Listen);
    }
    assert_eq!((r.guesses, r.rights), (1, 1));
}

#[test]
fn stale_timers_never_advance() {
    let mut r = Round::new(ALL[0], cfg(0), 1);
    let fx = r.handle(Event::Listen);
    let s = wait_seq(&fx).unwrap();
    r.handle(Event::Interrupted);
    assert_eq!(r.phase, Phase::Ready);
    r.handle(Event::Timer(s));
    assert_eq!(
        r.phase,
        Phase::Ready,
        "a timer from before the interruption is ignored"
    );
}

#[test]
fn background_mid_note_recovers_with_the_same_question() {
    let mut r = Round::new(ALL[0], cfg(0), 1);
    let q = r.q.clone();
    r.handle(Event::Listen);
    let fx = r.handle(Event::Interrupted);
    assert!(fx.contains(&Effect::StopAudio));
    assert_eq!((r.phase, &r.q), (Phase::Ready, &q));
    hear(&mut r); // and play continues
}

#[test]
fn audio_failure_never_opens_answers_and_offers_retry() {
    let mut r = Round::new(ALL[0], cfg(0), 1);
    let fx = r.handle(Event::Listen);
    r.handle(Event::AudioFailed);
    assert!(r.retry && !r.accepts_answer());
    r.handle(Event::Timer(wait_seq(&fx).unwrap()));
    assert_eq!(r.phase, Phase::Ready);
    hear(&mut r);
    assert!(!r.retry);
}

#[test]
fn two_misses_replay_the_question() {
    let mut r = Round::new(ALL[0], cfg(0), 1);
    hear(&mut r);
    let fx = r.handle(Event::Answer(wrong_target(&r)));
    r.handle(Event::Timer(wait_seq(&fx).unwrap()));
    assert_eq!(r.phase, Phase::Awaiting);
    let fx = r.handle(Event::Answer(wrong_target(&r)));
    let fx2 = r.handle(Event::Timer(wait_seq(&fx).unwrap()));
    assert_eq!(r.phase, Phase::Question);
    assert!(fx2.iter().any(|e| matches!(e, Effect::Play(_))));
}

#[test]
fn replay_is_allowed_after_the_question_but_closes_answers_again() {
    let mut r = Round::new(ALL[0], cfg(0), 1);
    hear(&mut r);
    r.handle(Event::Listen);
    assert_eq!(r.phase, Phase::Question);
    assert!(!r.accepts_answer());
}

#[test]
fn auto_next_off_waits_for_listen() {
    let mut c = cfg(0);
    c.auto_next = false;
    let mut r = Round::new(ALL[0], c, 1);
    hear(&mut r);
    let fx = r.handle(Event::Answer(r.q.answer[0]));
    r.handle(Event::Timer(wait_seq(&fx).unwrap()));
    assert_eq!(r.phase, Phase::Ready);
}

#[test]
fn two_choice_answers_never_run_four_times() {
    let (i, g) = by_slug("birdfish").unwrap();
    let mut r = Round::new(g, cfg(i), 99);
    let mut answers = vec![];
    for _ in 0..300 {
        answers.push(r.q.answer[0]);
        hear(&mut r);
        let fx = r.handle(Event::Answer(r.q.answer[0]));
        r.handle(Event::Timer(wait_seq(&fx).unwrap()));
        r.handle(Event::Interrupted);
    }
    assert!(answers.windows(4).all(|w| !w.iter().all(|&a| a == w[0])));
    assert!(answers.contains(&0) && answers.contains(&1));
}

#[test]
fn echo_tracks_progress_and_a_slip_starts_over() {
    let (i, g) = by_slug("echo").unwrap();
    let mut r = Round::new(g, cfg(i), 5);
    hear(&mut r);
    let first = r.q.answer[0];
    let fx = r.handle(Event::Answer(first));
    let total = r.q.answer.len();
    assert!(fx.contains(&Effect::Progress { done: 1, total }));
    assert_eq!(r.phase, Phase::Awaiting);
    r.handle(Event::Answer(wrong_target(&r)));
    assert_eq!((r.phase, r.step), (Phase::Wrong, 0));
}

#[test]
fn glow_after_three_misses_only_when_enabled() {
    for assist in [false, true] {
        let mut c = cfg(0);
        c.tune.glow_miss = if assist { 3 } else { 0 };
        c.tune.replay = 0;
        let mut r = Round::new(ALL[0], c, 1);
        hear(&mut r);
        let mut last = vec![];
        for _ in 0..3 {
            last = r.handle(Event::Answer(wrong_target(&r)));
            r.handle(Event::Timer(wait_seq(&last).unwrap()));
        }
        let glow = last.iter().any(|e| matches!(e, Effect::Hint(Some(_))));
        assert_eq!(glow, assist);
    }
}

#[test]
fn age_two_default_first_round_of_every_game_is_errorless() {
    let s = Settings::default();
    assert_eq!(s.age, Age::Two);
    for (i, g) in ALL
        .iter()
        .enumerate()
        .filter(|(_, g)| !g.free() && !g.beat())
    {
        let mut r = Round::new(*g, s.round_cfg(i), 7);
        assert!(r.targets().len() <= 2, "{}", g.slug());
        let fx = r.handle(Event::Listen);
        let fx = r.handle(Event::Timer(wait_seq(&fx).unwrap()));
        assert!(
            fx.contains(&Effect::Hint(Some(r.q.answer[0]))),
            "{}",
            g.slug()
        );
        let other = *r.targets().iter().find(|&&t| t != r.q.answer[0]).unwrap();
        r.handle(Event::Answer(other));
        assert_eq!((r.wrongs, r.phase), (0, Phase::Awaiting), "{}", g.slug());
        for &a in &r.q.answer.clone() {
            r.handle(Event::Answer(a));
        }
        assert_eq!(r.phase, Phase::Right, "{}", g.slug());
    }
}

// Touch rows: the gate in front of every game.

#[test]
fn gate_twenty_px_drag_is_one_tap_and_hundred_px_swipe_is_none() {
    let mut g = TouchGate::new();
    g.down(1, 0, 0.0, 0.0, 0, true);
    g.r#move(1, 12.0, 16.0);
    assert_eq!(g.up(1, 120), Decision::Guess(0));
    g.down(1, 0, 0.0, 0.0, 200, true);
    assert_eq!(g.r#move(1, 0.0, 100.0), Decision::Cancelled);
    assert_eq!(g.up(1, 400), Decision::Ignore);
}

#[test]
fn gate_finger_resting_through_question_end_cannot_answer() {
    let mut r = Round::new(ALL[0], cfg(0), 1);
    let mut g = TouchGate::new();
    let fx = r.handle(Event::Listen);
    assert_eq!(
        g.down(1, r.q.answer[0], 0.0, 0.0, 0, r.accepts_answer()),
        Decision::Ignore
    );
    r.handle(Event::Timer(wait_seq(&fx).unwrap()));
    assert!(r.accepts_answer());
    assert_eq!(g.up(1, 2000), Decision::Ignore);
}
