//! Pitches, the solfège table and the sound description every game uses.
//! The web layer turns a [`Sound`] into Web Audio; nothing here touches a
//! browser.

/// One solfège degree of C major from C4.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Degree {
    /// Stable key, also the CSS colour token (`--do`, `--re`, ...).
    pub key: &'static str,
    /// Label shown on the bar.
    pub name: &'static str,
    /// Spelling that makes speech engines say it right.
    pub say: &'static str,
    /// MIDI note number (60 = C4).
    pub midi: u8,
}

/// Do..Ti plus high Do. Colours never change (spec §2.4).
pub const SCALE: [Degree; 8] = [
    Degree {
        key: "do",
        name: "Do",
        say: "Doh",
        midi: 60,
    },
    Degree {
        key: "re",
        name: "Re",
        say: "Ray",
        midi: 62,
    },
    Degree {
        key: "mi",
        name: "Mi",
        say: "Mee",
        midi: 64,
    },
    Degree {
        key: "fa",
        name: "Fa",
        say: "Fah",
        midi: 65,
    },
    Degree {
        key: "sol",
        name: "Sol",
        say: "Soh",
        midi: 67,
    },
    Degree {
        key: "la",
        name: "La",
        say: "Lah",
        midi: 69,
    },
    Degree {
        key: "ti",
        name: "Ti",
        say: "Tee",
        midi: 71,
    },
    Degree {
        key: "do",
        name: "Do",
        say: "Doh",
        midi: 72,
    },
];

/// Equal temperament, A4 = 440 Hz.
pub fn freq(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

/// Which synthesized voice plays a tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voice {
    /// The warm marimba-like bar: every question and answer.
    Bar,
    /// The bright, soft reward chime.
    Chime,
}

/// One scheduled tone, relative to the start of its [`Sound`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tone {
    pub midi: f32,
    /// Start offset in seconds.
    pub at: f32,
    /// Nominal length in seconds (natural decay over this time).
    pub dur: f32,
    /// Linear gain, 1.0 = the standard loudness every game shares.
    pub gain: f32,
    pub voice: Voice,
    /// Light this answer target while the tone sounds (Echo's easy levels).
    pub light: Option<u8>,
}

impl Tone {
    pub fn bar(midi: u8, at: f32, dur: f32) -> Self {
        Tone {
            midi: midi as f32,
            at,
            dur,
            gain: 1.0,
            voice: Voice::Bar,
            light: None,
        }
    }
    pub fn gain(mut self, g: f32) -> Self {
        self.gain = g;
        self
    }
    pub fn lit(mut self, target: u8) -> Self {
        self.light = Some(target);
        self
    }
}

/// A group of tones plus its total length (when the last tone has rung).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Sound {
    pub tones: Vec<Tone>,
    pub dur: f32,
}

impl Sound {
    /// Build from tones; the length is the latest tone end.
    pub fn of(tones: Vec<Tone>) -> Self {
        let dur = tones.iter().map(|t| t.at + t.dur).fold(0.0, f32::max);
        Sound { tones, dur }
    }

    /// Append the reward chime starting at `at`.
    pub fn with_chime(mut self, at: f32) -> Self {
        for (i, m) in [79u8, 84, 88].into_iter().enumerate() {
            self.tones.push(Tone {
                midi: m as f32,
                at: at + i as f32 * 0.09,
                dur: 0.6,
                gain: 0.32,
                voice: Voice::Chime,
                light: None,
            });
        }
        self.dur = self.dur.max(at + 0.18 + 0.6);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_pitch_table() {
        let want = [261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88];
        for (d, hz) in SCALE.iter().zip(want) {
            assert!((freq(d.midi as f32) - hz).abs() < 0.01, "{} {}", d.name, hz);
        }
    }

    #[test]
    fn sound_length_is_last_tone_end() {
        let s = Sound::of(vec![Tone::bar(60, 0.0, 1.0), Tone::bar(64, 0.5, 1.0)]);
        assert_eq!(s.dur, 1.5);
    }
}
