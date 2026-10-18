//! The toddler touch gate (spec §2.1), shared by every game. Pure logic
//! with its own tests; the web layer forwards raw pointer events.
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
    target: u8,
    start_x: f32,
    start_y: f32,
    moved_px: f32,
    start_ms: u32,
}

impl TouchGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// A pointer landed on target `target`. `guessing_open` is the game's
    /// `accepts_guess()` AT THIS MOMENT — this is what makes resting
    /// fingers unable to score: their `down` happened while false.
    pub fn down(
        &mut self,
        pointer: u64,
        target: u8,
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
            target,
            start_x: x,
            start_y: y,
            moved_px: 0.0,
            start_ms: now_ms,
        });
        Decision::Track
    }

    /// The tracked pointer moved. Other pointers are inert.
    pub fn r#move(&mut self, pointer: u64, x: f32, y: f32) -> Decision {
        let Some(a) = self.active.as_mut() else {
            return Decision::Ignore;
        };
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
        let Some(a) = self.active.take() else {
            return Decision::Ignore;
        };
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
            Decision::Guess(a.target) // the FIRST-TOUCHED target, always
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

/// Free play and Drum along: instruments, not answers. Each finger sounds
/// at contact, up to `max` fingers at once, like hands on a real
/// xylophone. Lifting or cancelling one finger only releases that finger;
/// notes ring out on their own and are never cut by another finger.
#[derive(Debug, Default)]
pub struct Fingers {
    max: usize,
    down: Vec<Finger>,
}

#[derive(Clone, Copy, Debug)]
struct Finger {
    pointer: u64,
    /// The bar under the finger now (None: between bars, after a slide).
    target: Option<u8>,
    x: f32,
    y: f32,
}

impl Fingers {
    pub fn new(max: u8) -> Self {
        Fingers {
            max: max.max(1) as usize,
            down: vec![],
        }
    }

    pub fn set_max(&mut self, max: u8) {
        self.max = max.max(1) as usize;
    }

    /// A finger landed on `target`: true if it sounds. Fingers beyond the
    /// limit are ignored until one lifts.
    pub fn down(&mut self, pointer: u64, target: u8, x: f32, y: f32) -> bool {
        // A repeated down for the same finger replaces it, never doubles.
        self.down.retain(|f| f.pointer != pointer);
        if self.down.len() >= self.max {
            return false;
        }
        self.down.push(Finger {
            pointer,
            target: Some(target),
            x,
            y,
        });
        true
    }

    /// Free play: a finger slid to (`x`, `y`). Entering a bar plays it,
    /// like a glissando; leaving one and coming back plays it again.
    /// Returns (the bar it left, the bar it entered). Hysteresis: the
    /// finger keeps its bar until it is [`SLIDE_EDGE_PX`] past the bar's
    /// edge, so jitter on a boundary never plays two notes.
    pub fn slide(&mut self, pointer: u64, x: f32, y: f32, bars: &Bars) -> (Option<u8>, Option<u8>) {
        let Some(f) = self.down.iter_mut().find(|f| f.pointer == pointer) else {
            return (None, None);
        };
        if let Some(c) = f.target {
            if bars.near(c, x, y, SLIDE_EDGE_PX) {
                return (None, None);
            }
        }
        let now = bars.hit(x, y);
        if now == f.target {
            return (None, None);
        }
        let left = f.target;
        f.target = now;
        (left, now)
    }

    /// Drum along: a finger moved. Past the swipe distance it lets go of
    /// the drum (returned, so it can rise if no other finger holds it).
    pub fn r#move(&mut self, pointer: u64, x: f32, y: f32) -> Option<u8> {
        let i = self.down.iter().position(|f| f.pointer == pointer)?;
        let f = self.down[i];
        if dist(f.x, f.y, x, y) > SWIPE_CANCEL_PX {
            self.down.remove(i);
            return f.target;
        }
        None
    }

    /// A finger lifted or was cancelled: the bar it held, if any.
    pub fn up(&mut self, pointer: u64) -> Option<u8> {
        let i = self.down.iter().position(|f| f.pointer == pointer)?;
        self.down.remove(i).target
    }

    /// Is any finger still holding `target`?
    pub fn holds(&self, target: u8) -> bool {
        self.down.iter().any(|f| f.target == Some(target))
    }

    pub fn count(&self) -> usize {
        self.down.len()
    }

    /// Page hidden, screen change: every finger let go.
    pub fn reset(&mut self) {
        self.down.clear();
    }
}

/// A sliding finger keeps its bar until this far past the bar's edge.
pub const SLIDE_EDGE_PX: f32 = 8.0;

/// Free play's bar rectangles in CSS pixels, measured once per layout
/// (screen entry, resize, rotation) so a pointermove is a few compares.
#[derive(Clone, Debug, Default)]
pub struct Bars {
    rects: Vec<(u8, [f32; 4])>,
}

impl Bars {
    /// `rects`: (target, [left, top, right, bottom]).
    pub fn new(rects: Vec<(u8, [f32; 4])>) -> Self {
        Bars { rects }
    }

    /// The bar strictly under (`x`, `y`), if any.
    pub fn hit(&self, x: f32, y: f32) -> Option<u8> {
        self.rects
            .iter()
            .find(|(_, r)| x >= r[0] && x < r[2] && y >= r[1] && y < r[3])
            .map(|(t, _)| *t)
    }

    /// Is (`x`, `y`) on bar `t` or within `edge` px of it?
    pub fn near(&self, t: u8, x: f32, y: f32, edge: f32) -> bool {
        self.rects.iter().any(|(id, r)| {
            *id == t && x >= r[0] - edge && x < r[2] + edge && y >= r[1] - edge && y < r[3] + edge
        })
    }

    /// Bar `t`'s rectangle.
    pub fn rect(&self, t: u8) -> Option<[f32; 4]> {
        self.rects.iter().find(|(id, _)| *id == t).map(|(_, r)| *r)
    }

    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
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
    fn fingers_sound_together_up_to_the_limit() {
        let mut f = Fingers::new(3);
        assert!(f.down(1, 0, 0.0, 0.0));
        assert!(f.down(2, 2, 0.0, 90.0));
        assert!(f.down(3, 4, 0.0, 180.0));
        assert!(!f.down(4, 5, 0.0, 270.0), "a fourth finger past the limit");
        assert_eq!(f.up(4), None, "the ignored finger holds nothing");
        assert_eq!(f.up(2), Some(2));
        assert!(f.down(5, 5, 0.0, 270.0), "room again once one lifts");
        assert_eq!(f.count(), 3);
    }

    #[test]
    fn one_finger_at_a_time_when_the_limit_is_one() {
        let mut f = Fingers::new(1);
        assert!(f.down(1, 0, 0.0, 0.0));
        assert!(!f.down(2, 1, 0.0, 0.0));
        assert_eq!(f.up(1), Some(0));
        assert!(f.down(2, 1, 0.0, 0.0));
        let mut z = Fingers::new(0);
        assert!(z.down(1, 0, 0.0, 0.0), "never below one");
    }

    #[test]
    fn lifting_or_cancelling_one_finger_never_releases_another() {
        let mut f = Fingers::new(5);
        f.down(1, 3, 0.0, 0.0);
        f.down(2, 3, 30.0, 0.0); // two fingers on the same bar
        f.down(3, 5, 0.0, 100.0);
        assert_eq!(f.up(1), Some(3));
        assert!(f.holds(3), "the other finger still holds the bar");
        assert_eq!(f.up(3), Some(5)); // pointercancel goes through up too
        assert!(f.holds(3) && !f.holds(5));
        assert_eq!(f.up(3), None, "a second up is a no-op");
        assert_eq!(f.r#move(2, 40.0, 0.0), None, "small slide keeps it");
        assert_eq!(f.r#move(2, 130.0, 0.0), Some(3), "a swipe lets go");
        assert!(!f.holds(3));
        f.down(7, 1, 0.0, 0.0);
        f.down(8, 2, 0.0, 0.0);
        f.reset();
        assert_eq!(f.count(), 0, "hidden page: nothing stuck");
        assert!(f.down(7, 1, 0.0, 0.0));
    }

    #[test]
    fn a_repeated_down_for_one_finger_never_takes_two_places() {
        let mut f = Fingers::new(2);
        assert!(f.down(1, 0, 0.0, 0.0));
        assert!(f.down(1, 2, 0.0, 0.0));
        assert_eq!(f.count(), 1);
        assert!(f.down(2, 4, 0.0, 0.0));
        assert_eq!(f.up(1), Some(2));
    }

    /// Five bars stacked like the portrait xylophone: 60 px tall, 10 px gaps.
    fn stack() -> Bars {
        Bars::new(
            (0..5u8)
                .map(|i| (i, [0.0, i as f32 * 70.0, 300.0, i as f32 * 70.0 + 60.0]))
                .collect(),
        )
    }

    #[test]
    fn sliding_plays_each_bar_entered_once_in_order() {
        let b = stack();
        let mut f = Fingers::new(5);
        assert!(f.down(1, 0, 150.0, 30.0));
        let mut played = vec![];
        let mut y = 30.0;
        while y < 340.0 {
            if let (_, Some(t)) = f.slide(1, 150.0, y, &b) {
                played.push(t);
            }
            y += 3.0;
        }
        assert_eq!(played, vec![1, 2, 3, 4], "bar 0 sounded at contact");
        assert!(f.holds(4) && !f.holds(0));
        assert_eq!(f.up(1), Some(4));
    }

    #[test]
    fn leaving_a_bar_and_coming_back_plays_it_again() {
        let b = stack();
        let mut f = Fingers::new(5);
        f.down(1, 1, 150.0, 100.0);
        assert_eq!(f.slide(1, 150.0, 140.0, &b), (Some(1), Some(2)));
        assert_eq!(f.slide(1, 150.0, 100.0, &b), (Some(2), Some(1)), "re-entry");
        // Off the side of the xylophone, then back on.
        assert_eq!(f.slide(1, 330.0, 100.0, &b), (Some(1), None));
        assert!(!f.holds(1));
        assert_eq!(f.up(1), None, "lifted off the bars");
        f.down(2, 1, 150.0, 100.0);
        f.slide(2, 330.0, 100.0, &b);
        assert_eq!(f.slide(2, 290.0, 100.0, &b), (None, Some(1)));
    }

    #[test]
    fn jitter_on_a_bar_edge_never_machine_guns_notes() {
        let b = stack();
        let mut f = Fingers::new(5);
        f.down(1, 0, 150.0, 55.0);
        let mut notes = 0;
        for i in 0..40 {
            // ±6 px around the bottom edge of bar 0 (y = 60) and the gap.
            let y = 60.0 + if i % 2 == 0 { -6.0 } else { 6.0 };
            if f.slide(1, 150.0, y, &b).1.is_some() {
                notes += 1;
            }
        }
        assert_eq!(notes, 0);
        assert!(f.holds(0), "still on its bar");
        // Touching bars with no gap: a 4 px wobble across the seam keeps one.
        let seam = Bars::new(vec![
            (0, [0.0, 0.0, 100.0, 50.0]),
            (1, [0.0, 50.0, 100.0, 100.0]),
        ]);
        let mut g = Fingers::new(5);
        g.down(1, 0, 50.0, 48.0);
        for i in 0..20 {
            let y = if i % 2 == 0 { 52.0 } else { 48.0 };
            assert_eq!(g.slide(1, 50.0, y, &seam), (None, None));
        }
        assert_eq!(
            g.slide(1, 50.0, 60.0, &seam),
            (Some(0), Some(1)),
            "a real move still plays"
        );
    }

    #[test]
    fn two_fingers_slide_independently() {
        let b = stack();
        let mut f = Fingers::new(2);
        f.down(1, 0, 50.0, 30.0);
        f.down(2, 4, 250.0, 310.0);
        assert!(!f.down(3, 2, 150.0, 170.0), "a third finger past the limit");
        assert_eq!(
            f.slide(3, 150.0, 240.0, &b),
            (None, None),
            "it never slides in"
        );
        assert_eq!(f.slide(1, 50.0, 100.0, &b), (Some(0), Some(1)));
        assert_eq!(f.slide(2, 250.0, 240.0, &b), (Some(4), Some(3)));
        assert_eq!(f.slide(2, 250.0, 170.0, &b), (Some(3), Some(2)));
        assert_eq!(
            f.slide(1, 50.0, 170.0, &b),
            (Some(1), Some(2)),
            "both on one bar"
        );
        assert_eq!(f.up(2), Some(2));
        assert!(f.holds(2), "the other finger still holds it");
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
