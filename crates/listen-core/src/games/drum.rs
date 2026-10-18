//! Drum along: a bear drums a steady beat and the child drums with it.
//! Every touch sounds at once. At the easiest step every drum counts; on
//! harder steps a drum counts when it lands near the bear's beat.
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
    /// Counts when within this share of `ioi` of a beat (None: any drum).
    pub window: Option<f32>,
    /// Beats the bear plays alone first, as a model.
    pub lead_in: u32,
    /// Counted drums that finish a round.
    pub goal: u32,
    /// Beats the bear plays in all before resting.
    pub beats: u32,
}

pub const BEATS: [Beat; 3] = [
    Beat {
        ioi: 0.5,
        window: None,
        lead_in: 4,
        goal: 8,
        beats: 16,
    },
    Beat {
        ioi: 0.6,
        window: Some(0.25),
        lead_in: 4,
        goal: 8,
        beats: 24,
    },
    Beat {
        ioi: 0.5,
        window: Some(0.18),
        lead_in: 4,
        goal: 10,
        beats: 28,
    },
];

/// Drums closer together than this count once (a hand bouncing on glass).
pub const MIN_TAP_GAP: f32 = 0.15;

pub fn beat(step: u8) -> Beat {
    BEATS[(step as usize).min(BEATS.len() - 1)]
}

/// One round of drumming, timed on the audio clock from the first beat.
#[derive(Clone, Debug)]
pub struct DrumRun {
    pub beat: Beat,
    pub t0: f64,
    pub hits: u32,
    last_tap: Option<f64>,
    counted: Vec<i64>,
}

impl DrumRun {
    pub fn new(beat: Beat, t0: f64) -> DrumRun {
        DrumRun {
            beat,
            t0,
            hits: 0,
            last_tap: None,
            counted: vec![],
        }
    }

    pub fn beat_time(&self, n: u32) -> f64 {
        self.t0 + n as f64 * self.beat.ioi as f64
    }

    /// When the bear stops.
    pub fn end(&self) -> f64 {
        self.beat_time(self.beat.beats) + 0.3
    }

    /// A drum at audio time `t`. Returns whether it counted.
    pub fn tap(&mut self, t: f64) -> bool {
        if let Some(l) = self.last_tap {
            if t - l < MIN_TAP_GAP as f64 {
                return false;
            }
        }
        self.last_tap = Some(t);
        if self.done() {
            return false;
        }
        let ok = match self.beat.window {
            None => true,
            Some(w) => {
                let ioi = self.beat.ioi as f64;
                let n = ((t - self.t0) / ioi).round() as i64;
                let off = (t - (self.t0 + n as f64 * ioi)).abs();
                let fresh = !self.counted.contains(&n);
                let in_play = n >= self.beat.lead_in as i64 && n < self.beat.beats as i64;
                if off <= w as f64 * ioi && fresh && in_play {
                    self.counted.push(n);
                    true
                } else {
                    false
                }
            }
        };
        if ok {
            self.hits += 1;
        }
        ok
    }

    pub fn done(&self) -> bool {
        self.hits >= self.beat.goal
    }

    /// The bear's beats, lit on the bear.
    pub fn sound(&self) -> Sound {
        Sound::of(
            (0..self.beat.beats)
                .map(|n| {
                    let lead = n < self.beat.lead_in;
                    Tone::bar(50, n as f32 * self.beat.ioi, 0.3)
                        .voiced(Voice::Drum)
                        .gain(if lead { 0.9 } else { 0.6 })
                        .lit(BEAR)
                })
                .collect(),
        )
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

    #[test]
    fn the_easiest_step_counts_every_drum_but_not_bounces() {
        let mut r = DrumRun::new(beat(0), 10.0);
        assert!(r.tap(10.03));
        assert!(!r.tap(10.1), "a bounce 70 ms later counts once");
        assert!(r.tap(10.77), "off the beat still counts at step 0");
        assert_eq!(r.hits, 2);
    }

    #[test]
    fn judged_steps_count_drums_near_a_beat_once_each_after_the_lead_in() {
        let b = beat(1);
        let mut r = DrumRun::new(b, 0.0);
        assert!(!r.tap(b.ioi as f64), "during the bear's lead-in");
        let n4 = 4.0 * b.ioi as f64;
        assert!(r.tap(n4 + 0.05));
        assert!(!r.tap(n4 + 0.25), "same beat twice");
        let n5 = 5.0 * b.ioi as f64;
        assert!(r.tap(n5 - 0.1));
        let n6 = 6.0 * b.ioi as f64;
        assert!(!r.tap(n6 + 0.5 * b.ioi as f64 - 0.01), "half a beat off");
        assert_eq!(r.hits, 2);
    }

    #[test]
    fn a_round_ends_at_the_goal_and_the_bear_leaves_room_to_reach_it() {
        for b in BEATS {
            assert!(b.beats >= b.lead_in + b.goal * 3 / 2, "{b:?}");
            assert!((0.4..=0.6).contains(&b.ioi), "near children's own tempo");
            let mut r = DrumRun::new(b, 0.0);
            for n in b.lead_in..b.beats {
                r.tap(r.beat_time(n));
            }
            assert!(r.done());
            assert_eq!(r.hits, b.goal);
            assert_eq!(r.sound().tones.len() as u32, b.beats);
        }
    }
}
