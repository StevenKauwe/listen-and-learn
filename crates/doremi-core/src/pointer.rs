//! The toddler touch gate (spec §3). This is the highest-priority
//! engineering requirement, so it is pure logic with its own tests.
//!
//! What it encodes:
//! * A tap is judged on the button **touched first**, never the button
//!   under the finger at release.
//! * Small-to-moderate drift (default ≤ 35 CSS px) still counts as a tap.
//! * Long presses count. Quick taps count. Release position is free.
//! * Big movement (default > 70 px) is an intentional swipe: cancelled.
//! * Between the two: still a tap if it was slow and deliberate (held
//!   ≥ 300 ms), otherwise a swipe.
//! * Only ONE pointer may ever be a guess candidate. Extra fingers are
//!   passively ignored until the interaction ends.
//! * A finger already down when guessing opens can never score: only a
//!   *fresh* `down` in the accepting phase can (the UI checks
//!   `accepts_guess()` at `down` time and tells the gate).
//! * `pointercancel` / suspension clears everything. No stuck pointers.

/// Distance in CSS pixels a toddler may wander and still be a tap.
pub const TAP_TOLERANCE_PX: f32 = 35.0;
/// Beyond this it was clearly a swipe.
pub const SWIPE_CANCEL_PX: f32 = 70.0;
/// A slow, deliberate drag between the thresholds counts as a press.
pub const SLOW_PRESS_MS: u32 = 300;

/// What the UI should do with a pointer event it forwarded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Nothing (finger down outside buttons, extra finger, or guessing
    /// closed). No state to track.
    Ignore,
    /// Tracking an interaction that may become a guess. Keep sending
    /// moves for this pointer id.
    Track,
    /// Interaction cancelled (too much movement). No guess, ever.
    Cancelled,
    /// A deliberate tap completed: exactly one guess for this button.
    Guess(u8),
}

/// Tracks at most one candidate interaction. Pointer ids are opaque.
#[derive(Default, Debug)]
pub struct TouchGate {
    active: Option<Active>,
}

#[derive(Clone, Copy, Debug)]
struct Active {
    pointer: u64,
    button: u8,
    start_x: f32,
    start_y: f32,
    moved_px: f32,
    start_ms: u32,
}

impl TouchGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// A pointer landed on button `button`. `guessing_open` is the game's
    /// `accepts_guess()` AT THIS MOMENT — this is what makes resting
    /// fingers unable to score: their `down` happened while false.
    pub fn down(
        &mut self,
        pointer: u64,
        button: u8,
        x: f32,
        y: f32,
        now_ms: u32,
        guessing_open: bool,
    ) -> Decision {
        // Extra finger while one is live: never a second candidate.
        if let Some(a) = self.active {
            if a.pointer == pointer {
                // Duplicate down for the same finger (shouldn't happen);
                // reset it to be safe rather than stuck.
                self.active = None;
            } else {
                return Decision::Ignore;
            }
        }
        if !guessing_open {
            return Decision::Ignore; // never queue; never remember
        }
        self.active = Some(Active {
            pointer,
            button,
            start_x: x,
            start_y: y,
            moved_px: 0.0,
            start_ms: now_ms,
        });
        Decision::Track
    }

    /// The tracked pointer moved. Other pointers are inert.
    pub fn r#move(&mut self, pointer: u64, x: f32, y: f32) -> Decision {
        let Some(a) = self.active.as_mut() else { return Decision::Ignore };
        if a.pointer != pointer {
            return Decision::Ignore;
        }
        a.moved_px = a.moved_px.max(dist(a.start_x, a.start_y, x, y));
        if a.moved_px > SWIPE_CANCEL_PX {
            self.active = None;
            return Decision::Cancelled;
        }
        Decision::Track
    }

    /// The tracked pointer lifted: the moment of truth.
    pub fn up(&mut self, pointer: u64, now_ms: u32) -> Decision {
        let Some(a) = self.active.take() else { return Decision::Ignore };
        if a.pointer != pointer {
            // A different finger lifted while one was tracked: put it
            // back, the real one may still land.
            self.active = Some(a);
            return Decision::Ignore;
        }
        let held = now_ms.saturating_sub(a.start_ms);
        if a.moved_px <= TAP_TOLERANCE_PX
            || (a.moved_px <= SWIPE_CANCEL_PX && held >= SLOW_PRESS_MS)
        {
            Decision::Guess(a.button) // the FIRST-TOUCHED button, always
        } else {
            Decision::Cancelled
        }
    }

    /// pointercancel / suspension / phase change: forget everything so
    /// nothing can be stuck.
    pub fn cancel(&mut self, pointer: u64) -> Decision {
        if let Some(a) = self.active {
            if a.pointer == pointer {
                self.active = None;
                return Decision::Cancelled;
            }
        }
        Decision::Ignore
    }

    /// Drop all pointers (app suspension, state transitions).
    pub fn reset(&mut self) {
        self.active = None;
    }

    pub fn is_tracking(&self) -> bool {
        self.active.is_some()
    }
}

fn dist(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drift_twenty_px_is_one_guess() {
        let mut g = TouchGate::new();
        assert_eq!(g.down(1, 0, 100.0, 200.0, 0, true), Decision::Track);
        g.r#move(1, 112.0, 208.0); // ~14 px
        assert_eq!(g.r#move(1, 108.0, 215.0), Decision::Track); // ~20 px
        assert_eq!(g.up(1, 900), Decision::Guess(0));
    }

    #[test]
    fn long_press_counts() {
        let mut g = TouchGate::new();
        g.down(1, 2, 0.0, 0.0, 0, true);
        g.r#move(1, 5.0, 4.0);
        assert_eq!(g.up(1, 2000), Decision::Guess(2)); // 2-second hold
    }

    #[test]
    fn slow_deliberate_drag_between_thresholds_is_a_press() {
        let mut g = TouchGate::new();
        g.down(1, 1, 0.0, 0.0, 0, true);
        g.r#move(1, 50.0, 0.0); // 50 px: between 35 and 70
        assert_eq!(g.up(1, 500), Decision::Guess(1)); // held 500 ms
    }

    #[test]
    fn fast_big_swipe_is_cancelled() {
        let mut g = TouchGate::new();
        g.down(1, 0, 0.0, 0.0, 0, true);
        assert_eq!(g.r#move(1, 100.0, 0.0), Decision::Cancelled);
        assert_eq!(g.up(1, 80), Decision::Ignore); // nothing pending
        assert!(!g.is_tracking());
    }

    #[test]
    fn sliding_over_neighbors_never_switches_buttons() {
        let mut g = TouchGate::new();
        g.down(1, 0, 0.0, 0.0, 0, true);
        g.r#move(1, 20.0, 5.0); // drifted over the next button visually
        assert_eq!(g.up(1, 400), Decision::Guess(0)); // first-touched wins
    }

    #[test]
    fn second_finger_is_inert() {
        let mut g = TouchGate::new();
        g.down(1, 0, 0.0, 0.0, 0, true);
        assert_eq!(g.down(2, 1, 50.0, 50.0, 10, true), Decision::Ignore);
        assert_eq!(g.r#move(2, 300.0, 0.0), Decision::Ignore); // even wild
        assert_eq!(g.up(2, 100), Decision::Ignore);
        assert_eq!(g.up(1, 150), Decision::Guess(0)); // first finger wins
    }

    #[test]
    fn finger_resting_before_guessing_opens_cannot_score() {
        let mut g = TouchGate::new();
        // Down while guessing was closed (mystery note playing):
        assert_eq!(g.down(1, 0, 0.0, 0.0, 0, false), Decision::Ignore);
        // Guessing opens; the finger lifts:
        assert_eq!(g.up(1, 500), Decision::Ignore); // never queued
        // A NEW touch is required and works:
        assert_eq!(g.down(1, 0, 0.0, 0.0, 600, true), Decision::Track);
        assert_eq!(g.up(1, 700), Decision::Guess(0));
    }

    #[test]
    fn pointercancel_clears_without_scoring() {
        let mut g = TouchGate::new();
        g.down(1, 3, 0.0, 0.0, 0, true);
        assert_eq!(g.cancel(1), Decision::Cancelled);
        assert_eq!(g.up(1, 100), Decision::Ignore);
        assert!(!g.is_tracking());
    }

    #[test]
    fn rapid_retouches_each_score_exactly_once() {
        let mut g = TouchGate::new();
        for i in 0..10 {
            let t = i * 300;
            g.down(7, 1, 0.0, 0.0, t, true);
            g.r#move(7, 10.0, 0.0);
            assert_eq!(g.up(7, t + 100), Decision::Guess(1));
            assert_eq!(g.up(7, t + 110), Decision::Ignore); // dedup
        }
    }
}
