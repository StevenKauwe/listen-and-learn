//! Adult settings (spec §2.6), with a tiny line format for localStorage.
//! Anything unreadable falls back to defaults; every value is clamped.

use crate::games::{clamp_level, Knobs, ALL};
use crate::round::RoundCfg;

/// Hold time to open the settings sheet.
pub const HOLD_SETTINGS_MS: u32 = 2000;
/// Hold time to leave a game for the home screen.
pub const HOLD_HOME_MS: u32 = 1500;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// Level per game, in [`ALL`] order.
    pub levels: Vec<u8>,
    pub volume: f32,
    pub note_len: f32,
    pub speak: bool,
    pub auto_next: bool,
    pub cheer: bool,
    pub assist: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            levels: ALL.iter().map(|g| g.default_level()).collect(),
            volume: 0.8,
            note_len: 1.2,
            speak: false,
            auto_next: true,
            cheer: true,
            assist: false,
        }
    }
}

impl Settings {
    pub fn sanitized(mut self) -> Self {
        let defaults = Settings::default();
        self.levels.resize(ALL.len(), 0);
        for (i, g) in ALL.iter().enumerate() {
            self.levels[i] = clamp_level(*g, self.levels[i]);
        }
        self.volume = if self.volume.is_finite() {
            self.volume.clamp(0.0, 1.0)
        } else {
            defaults.volume
        };
        self.note_len = if self.note_len.is_finite() {
            self.note_len.clamp(0.8, 2.0)
        } else {
            defaults.note_len
        };
        self
    }

    pub fn round_cfg(&self, game_index: usize) -> RoundCfg {
        RoundCfg {
            knobs: Knobs {
                level: self.levels[game_index],
                note_len: self.note_len,
            },
            auto_next: self.auto_next,
            cheer: self.cheer,
            speak: self.speak,
            assist: self.assist,
            replay_after: 2,
        }
    }

    /// `key=value` lines; levels as `lv.<slug>=n`.
    pub fn encode(&self) -> String {
        let mut s = format!(
            "v=1\nvolume={}\nnote_len={}\nspeak={}\nauto_next={}\ncheer={}\nassist={}\n",
            self.volume,
            self.note_len,
            self.speak as u8,
            self.auto_next as u8,
            self.cheer as u8,
            self.assist as u8
        );
        for (g, l) in ALL.iter().zip(&self.levels) {
            s.push_str(&format!("lv.{}={}\n", g.slug(), l));
        }
        s
    }

    pub fn decode(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let flag = v == "1";
            match k {
                "volume" => s.volume = v.parse().unwrap_or(s.volume),
                "note_len" => s.note_len = v.parse().unwrap_or(s.note_len),
                "speak" => s.speak = flag,
                "auto_next" => s.auto_next = flag,
                "cheer" => s.cheer = flag,
                "assist" => s.assist = flag,
                _ => {
                    if let Some(slug) = k.strip_prefix("lv.") {
                        if let (Some(i), Ok(n)) =
                            (ALL.iter().position(|g| g.slug() == slug), v.parse())
                        {
                            s.levels[i] = n;
                        }
                    }
                }
            }
        }
        s.sanitized()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let mut s = Settings {
            volume: 0.5,
            speak: true,
            ..Settings::default()
        };
        s.levels[0] = 3;
        assert_eq!(Settings::decode(&s.encode()), s);
    }

    #[test]
    fn garbage_and_out_of_range_values_fall_back_or_clamp() {
        let s = Settings::decode("volume=9\nnote_len=NaN\nlv.notes=99\nlv.nope=1\n???\n");
        assert_eq!(s.volume, 1.0);
        assert_eq!(s.note_len, 1.2);
        assert_eq!(s.levels[0], 3);
        assert_eq!(Settings::decode(""), Settings::default());
    }

    #[test]
    fn spec_defaults() {
        let s = Settings::default();
        assert_eq!(s.levels[0], 1, "Do Re Mi starts on Standard");
        assert_eq!(s.note_len, 1.2);
        assert!(!s.speak, "spoken names default off");
    }
}
