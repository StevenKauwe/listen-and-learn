//! Drum along: a bear drums a steady beat and the child drums with it.
//! Every touch sounds at once, and the bear drums on for as long as the
//! child does. For older children an optional streak counts drums in a
//! row that land near the bear's beat; a drum between beats only starts
//! the count again, silently.
//!
//! Children synchronise best with a (here: animated) partner and near
//! their own spontaneous tempo of about 400–500 ms (Kirschner & Tomasello
//! 2009; Provasi & Bobin-Bègue 2003, as reported in Bobin-Bègue et al.
//! 2014). Many 3–5-year-olds cannot yet keep a beat (Bonacina et al. 2021),
//! so nothing here is ever "wrong".

use super::{Game, Knobs, Question};
use crate::music::{Sound, Tone, Voice};
use crate::rng::Rng;

pub struct Drum;

/// The drum the child plays; the bear is target 1 (never touchable).
pub const DRUM: u8 = 0;
pub const BEAR: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beat {
    /// Seconds between the bear's beats.
    pub ioi: f32,
    /// The streak counts a drum within this share of `ioi` of a beat.
    pub window: f32,
    /// Beats the bear plays alone first, as a model (a little louder).
    pub lead_in: u32,
}

/// Rung 0 (ages 2–3) is drumming together; its window only matters if a
/// grown-up turns the streak on. Rungs 1–2 are the age 4 and 5+ beats.
pub const BEATS: [Beat; 3] = [
    Beat {
        ioi: 0.5,
        window: 0.3,
        lead_in: 4,
    },
    Beat {
        ioi: 0.6,
        window: 0.25,
        lead_in: 4,
    },
    Beat {
        ioi: 0.5,
        window: 0.18,
        lead_in: 4,
    },
];

/// Drums closer together than this count once (a hand bouncing on glass).
pub const MIN_TAP_GAP: f32 = 0.15;
/// Touches closer together than this make one drum sound (a palm, or two
/// fingers landing as one hand).
pub const SOUND_GAP: f32 = 0.04;
/// The bear keeps drumming as long as the child does; after this long with
/// no drum it rests until Listen.
pub const IDLE_REST_S: f32 = 30.0;

pub fn beat(step: u8) -> Beat {
    BEATS[(step as usize).min(BEATS.len() - 1)]
}

/// How a drum landed, for the streak (when it is on).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tap {
    /// A bounce, a second drum on the same beat, or the bear's lead-in:
    /// changes nothing.
    Neutral,
    /// On a beat; the streak is now this long.
    OnBeat(u32),
    /// Between beats: the streak quietly starts again from 0.
    Off,
}

/// The bear's endless beat, timed on the audio clock from the first beat.
/// Nothing ends it but the child stopping (or leaving): no goal, no reset.
#[derive(Clone, Debug)]
pub struct DrumRun {
    pub beat: Beat,
    pub t0: f64,
    /// Consecutive on-beat drums.
    pub streak: u32,
    pub best: u32,
    last_beat: Option<i64>,
    last_tap: Option<f64>,
    last_drum: f64,
    scheduled: u32,
}

impl DrumRun {
    pub fn new(beat: Beat, t0: f64) -> DrumRun {
        DrumRun {
            beat,
            t0,
            streak: 0,
            best: 0,
            last_beat: None,
            last_tap: None,
            last_drum: t0,
            scheduled: 0,
        }
    }

    pub fn beat_time(&self, n: u32) -> f64 {
        self.t0 + n as f64 * self.beat.ioi as f64
    }

    /// Beats not yet scheduled that start before `until`: (index, time).
    /// The web layer calls this a little ahead of the audio clock, so the
    /// bear never stops while the child drums.
    pub fn due(&mut self, until: f64) -> Vec<(u32, f64)> {
        let mut out = vec![];
        while self.beat_time(self.scheduled) < until {
            out.push((self.scheduled, self.beat_time(self.scheduled)));
            self.scheduled += 1;
        }
        out
    }

    /// The bear rests when nobody has drummed for a while.
    pub fn resting(&self, now: f64) -> bool {
        now - self.last_drum > IDLE_REST_S as f64
    }

    /// A drum heard at audio time `t` (already corrected for output
    /// latency, see [`heard_at`]).
    pub fn tap(&mut self, t: f64) -> Tap {
        self.last_drum = self.last_drum.max(t);
        if let Some(l) = self.last_tap {
            if t - l < MIN_TAP_GAP as f64 {
                return Tap::Neutral;
            }
        }
        self.last_tap = Some(t);
        let ioi = self.beat.ioi as f64;
        let n = ((t - self.t0) / ioi).round() as i64;
        if n < self.beat.lead_in as i64 {
            return Tap::Neutral;
        }
        let off = (t - (self.t0 + n as f64 * ioi)).abs();
        if off > self.beat.window as f64 * ioi {
            self.streak = 0;
            return Tap::Off;
        }
        if self.last_beat == Some(n) {
            return Tap::Neutral;
        }
        self.streak = if self.last_beat == Some(n - 1) {
            self.streak + 1
        } else {
            1
        };
        self.last_beat = Some(n);
        self.best = self.best.max(self.streak);
        Tap::OnBeat(self.streak)
    }

    /// Beat `n` of the bear, lit on the bear.
    pub fn beat_sound(&self, n: u32) -> Sound {
        let lead = n < self.beat.lead_in;
        Sound::of(vec![Tone::bar(50, 0.0, 0.3)
            .voiced(Voice::Drum)
            .gain(if lead { 0.9 } else { 0.6 })
            .lit(BEAR)])
    }
}

/// When a touch was heard, on the audio clock. The child drums to the
/// beat they *hear*, which leaves the speaker `output latency` after the
/// audio clock schedules it, and the touch happened `event age` before
/// the handler ran. With an output timestamp (`ctx`, `perf` from
/// `getOutputTimestamp`, the audio time leaving the speaker at a
/// performance time) the mapping is exact; otherwise fall back to the
/// current time less the reported latencies.
pub fn heard_at(
    ctx_now: f64,
    perf_now_ms: f64,
    event_ms: f64,
    stamp: Option<(f64, f64)>,
    latency: f64,
) -> f64 {
    let age = ((perf_now_ms - event_ms) / 1000.0).clamp(0.0, 0.5);
    match stamp {
        Some((ctx, perf)) if ctx > 0.0 && perf > 0.0 => ctx + (event_ms - perf) / 1000.0,
        _ => ctx_now - age - latency.clamp(0.0, 0.5),
    }
}

/// The child's drum: a higher, brighter drum than the bear's.
pub fn drum_sound() -> Sound {
    Sound::of(vec![Tone::bar(57, 0.0, 0.3).voiced(Voice::Drum)])
}

impl Game for Drum {
    fn slug(&self) -> &'static str {
        "drum"
    }
    fn title(&self) -> &'static str {
        "Drum along"
    }
    fn steps(&self) -> &'static [&'static str] {
        &["Drum with me", "Keep the beat (slow)", "Keep the beat"]
    }
    fn targets(&self, _step: u8) -> Vec<u8> {
        vec![DRUM]
    }
    fn question(&self, _k: Knobs, _rng: &mut Rng, _prev: Option<&Question>) -> Question {
        Question {
            answer: vec![],
            sound: Sound::default(),
            name: None,
            sing: None,
        }
    }
    fn answer_sound(&self, _k: Knobs, _target: u8) -> Sound {
        drum_sound()
    }
    fn hint(&self) -> Sound {
        let mut t: Vec<Tone> = (0..3)
            .map(|n| Tone::bar(50, n as f32 * 0.5, 0.3).voiced(Voice::Drum))
            .collect();
        t.push(Tone::bar(57, 0.25, 0.3).voiced(Voice::Drum).gain(0.6));
        Sound::of(t)
    }
    fn beat(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(r: &DrumRun, n: u32, off: f64) -> f64 {
        r.beat_time(n) + off
    }

    #[test]
    fn on_beat_drums_build_a_streak_and_an_off_beat_drum_quietly_restarts_it() {
        let b = beat(1);
        let mut r = DrumRun::new(b, 0.0);
        assert_eq!(r.tap(on(&r, 1, 0.0)), Tap::Neutral, "the bear's lead-in");
        assert_eq!(r.tap(on(&r, 4, 0.05)), Tap::OnBeat(1));
        assert_eq!(r.tap(on(&r, 4, 0.2)), Tap::Neutral, "same beat twice");
        assert_eq!(r.tap(on(&r, 5, -0.1)), Tap::OnBeat(2));
        assert_eq!(r.tap(on(&r, 6, 0.0)), Tap::OnBeat(3));
        assert_eq!(r.tap(on(&r, 6, 0.3)), Tap::Off, "half a beat off");
        assert_eq!(r.streak, 0);
        assert_eq!(r.tap(on(&r, 7, 0.0)), Tap::OnBeat(1));
        assert_eq!(r.tap(on(&r, 9, 0.0)), Tap::OnBeat(1), "a skipped beat");
        assert_eq!(r.best, 3);
    }

    #[test]
    fn bounces_count_once_and_never_break_a_streak() {
        let mut r = DrumRun::new(beat(2), 0.0);
        assert_eq!(r.tap(on(&r, 4, 0.0)), Tap::OnBeat(1));
        assert_eq!(r.tap(on(&r, 4, 0.07)), Tap::Neutral);
        assert_eq!(r.tap(on(&r, 5, 0.0)), Tap::OnBeat(2));
    }

    #[test]
    fn windows_follow_the_age_rungs() {
        // Age 4: ±25 % of 600 ms; 5+: ±18 % of 500 ms (docs/research.md).
        for (step, inside, outside) in [(1u8, 0.145, 0.16), (2, 0.085, 0.095), (0, 0.145, 0.16)] {
            let mut r = DrumRun::new(beat(step), 0.0);
            assert_eq!(r.tap(on(&r, 4, inside)), Tap::OnBeat(1), "step {step}");
            assert_eq!(r.tap(on(&r, 5, -outside)), Tap::Off, "step {step}");
        }
        for b in BEATS {
            assert!((0.4..=0.6).contains(&b.ioi), "near children's own tempo");
        }
    }

    #[test]
    fn the_bear_never_stops_while_the_child_drums_and_rests_when_they_stop() {
        let b = beat(0);
        let mut r = DrumRun::new(b, 0.0);
        let mut n = 0;
        let mut t = 0.0;
        while t < 300.0 {
            for (i, at) in r.due(t + 1.0) {
                assert_eq!(i, n, "every beat once, in order");
                assert!((at - r.beat_time(i)).abs() < 1e-9);
                n += 1;
            }
            if ((t * 4.0) as u32).is_multiple_of(4) {
                r.tap(t);
            }
            assert!(!r.resting(t), "drumming at {t}");
            t += 0.25;
        }
        assert!(n as f64 >= 300.0 / b.ioi as f64, "five minutes of beat");
        assert!(r.resting(t + IDLE_REST_S as f64 + 1.0));
        assert_eq!(r.beat_sound(0).tones[0].light, Some(BEAR));
        assert!(r.beat_sound(0).tones[0].gain > r.beat_sound(9).tones[0].gain);
    }

    #[test]
    fn heard_time_corrects_for_output_latency_and_event_age() {
        // Output stamp: audio time 10.0 left the speaker at perf 5000 ms.
        let t = heard_at(10.2, 5200.0, 5180.0, Some((10.0, 5000.0)), 0.0);
        assert!((t - 10.18).abs() < 1e-9);
        // No stamp: now, less the touch's age, less the reported latency.
        let t = heard_at(10.2, 5200.0, 5180.0, None, 0.05);
        assert!((t - (10.2 - 0.02 - 0.05)).abs() < 1e-9);
        // Absurd values are bounded.
        let t = heard_at(10.0, 5000.0, 0.0, None, 9.0);
        assert!((t - 9.0).abs() < 1e-9);
    }
}
