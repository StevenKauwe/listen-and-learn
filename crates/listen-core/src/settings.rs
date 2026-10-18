//! Adult settings (spec §2.6), with a tiny line format for localStorage.
//! Anything unreadable falls back to defaults; every value is clamped.
//!
//! The simple control is the child's age: it sets every game's tune
//! (calib.rs). The "More" panel edits single tunes afterwards.

use crate::calib::{preset, Age, Tune};
use crate::games::{Knobs, ALL};
use crate::round::RoundCfg;

/// Hold time to open the settings sheet.
pub const HOLD_SETTINGS_MS: u32 = 2000;
/// Hold time to leave a game for the home screen.
pub const HOLD_HOME_MS: u32 = 1500;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub age: Age,
    /// Tune per game, in [`ALL`] order.
    pub tunes: Vec<Tune>,
    pub volume: f32,
    pub note_len: f32,
    /// Sing note names after a right answer, on their own pitch.
    pub sing: bool,
    pub auto_next: bool,
    pub cheer: bool,
    /// Right answers in a set before a rest (0 = no rest).
    pub set_len: u8,
}

/// Note-length range for the adult slider.
pub const NOTE_LEN: (f32, f32) = (0.8, 2.0);

impl Default for Settings {
    fn default() -> Self {
        let mut s = Settings {
            age: Age::Two,
            tunes: vec![],
            volume: 0.8,
            note_len: 1.2,
            sing: true,
            auto_next: true,
            cheer: true,
            set_len: 0,
        };
        s.set_age(Age::Two);
        s
    }
}

impl Settings {
    /// The simple control: one age sets every game.
    pub fn set_age(&mut self, age: Age) {
        self.age = age;
        self.tunes = ALL
            .iter()
            .map(|g| preset(age, g.slug()).clamped(g.steps().len()))
            .collect();
        self.set_len = age.set_len();
    }

    /// True when the tune of game `i` is not its age preset.
    pub fn custom(&self, i: usize) -> bool {
        let g = ALL[i];
        self.tunes[i] != preset(self.age, g.slug()).clamped(g.steps().len())
    }

    pub fn sanitized(mut self) -> Self {
        let defaults = Settings::default();
        if self.tunes.len() != ALL.len() {
            let mut t: Vec<Tune> = ALL.iter().map(|g| preset(self.age, g.slug())).collect();
            for (i, x) in self.tunes.iter().enumerate().take(ALL.len()) {
                t[i] = *x;
            }
            self.tunes = t;
        }
        for (i, g) in ALL.iter().enumerate() {
            self.tunes[i] = self.tunes[i].clamped(g.steps().len());
        }
        self.volume = if self.volume.is_finite() {
            self.volume.clamp(0.0, 1.0)
        } else {
            defaults.volume
        };
        self.note_len = if self.note_len.is_finite() {
            self.note_len.clamp(NOTE_LEN.0, NOTE_LEN.1)
        } else {
            defaults.note_len
        };
        self.set_len = self.set_len.min(30);
        self
    }

    pub fn round_cfg(&self, game_index: usize) -> RoundCfg {
        let tune = self.tunes[game_index];
        RoundCfg {
            knobs: Knobs {
                level: tune.lo,
                note_len: self.note_len,
            },
            tune,
            auto_next: self.auto_next,
            cheer: self.cheer,
            sing: self.sing,
            set_len: self.set_len,
        }
    }

    /// `key=value` lines; tunes as `t.<slug>=lo,hi,show,light,glow_miss,glow_wait,replay`.
    pub fn encode(&self) -> String {
        let mut s = format!(
            "v=2\nage={}\nvolume={}\nnote_len={}\nsing={}\nauto_next={}\ncheer={}\nset_len={}\n",
            self.age.years(),
            self.volume,
            self.note_len,
            self.sing as u8,
            self.auto_next as u8,
            self.cheer as u8,
            self.set_len,
        );
        for (g, t) in ALL.iter().zip(&self.tunes) {
            s.push_str(&format!(
                "t.{}={},{},{},{},{},{},{}\n",
                g.slug(),
                t.lo,
                t.hi,
                t.show,
                t.light as u8,
                t.glow_miss,
                t.glow_wait,
                t.replay
            ));
        }
        s
    }

    pub fn decode(text: &str) -> Settings {
        let mut s = Settings::default();
        // The age comes first: it sets the tunes that lines below override.
        for line in text.lines() {
            if let Some(v) = line.strip_prefix("age=") {
                if let Ok(y) = v.parse::<u8>() {
                    s.set_age(Age::from_years(y));
                }
            }
        }
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let flag = v == "1";
            match k {
                "volume" => s.volume = v.parse().unwrap_or(s.volume),
                "note_len" => s.note_len = v.parse().unwrap_or(s.note_len),
                "sing" => s.sing = flag,
                "auto_next" => s.auto_next = flag,
                "cheer" => s.cheer = flag,
                "set_len" => s.set_len = v.parse().unwrap_or(s.set_len),
                _ => {
                    let Some(slug) = k.strip_prefix("t.") else {
                        continue;
                    };
                    let Some(i) = ALL.iter().position(|g| g.slug() == slug) else {
                        continue;
                    };
                    let n: Vec<u8> = v.split(',').filter_map(|x| x.parse().ok()).collect();
                    if let [lo, hi, show, light, glow_miss, glow_wait, replay] = n[..] {
                        s.tunes[i] = Tune {
                            lo,
                            hi,
                            show,
                            light: light == 1,
                            glow_miss,
                            glow_wait,
                            replay,
                        };
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
            sing: false,
            ..Settings::default()
        };
        s.set_age(Age::Four);
        s.tunes[0].show = 5;
        s.tunes[2].hi = 3;
        assert_eq!(Settings::decode(&s.encode()), s);
    }

    #[test]
    fn garbage_and_out_of_range_values_fall_back_or_clamp() {
        let s = Settings::decode(
            "volume=9\nnote_len=NaN\nage=x\nt.notes=99,99,99,1,1,99,1\nt.nope=1\n???\n",
        );
        assert_eq!(s.volume, 1.0);
        assert_eq!(s.note_len, 1.2);
        assert_eq!(s.age, Age::Two);
        assert_eq!(s.tunes[0].lo as usize, ALL[0].steps().len() - 1);
        assert_eq!(Settings::decode(""), Settings::default());
    }

    #[test]
    fn version_one_settings_keep_volume_and_start_at_age_two() {
        let s = Settings::decode("v=1\nvolume=0.4\nspeak=1\nlv.notes=3\n");
        assert_eq!(s.volume, 0.4);
        assert_eq!(s.age, Age::Two);
        assert!(!s.custom(0));
    }

    #[test]
    fn defaults_are_age_two_with_sung_names() {
        let s = Settings::default();
        assert_eq!(s.age, Age::Two);
        assert!(s.sing, "sung names on: a model to sing back");
        assert_eq!(s.note_len, 1.2);
        assert!((0..ALL.len()).all(|i| !s.custom(i)));
    }

    #[test]
    fn choosing_an_age_sets_every_game_and_clears_custom_tunes() {
        let mut s = Settings::default();
        s.tunes[1].lo = 2;
        assert!(s.custom(1));
        s.set_age(Age::Three);
        assert!((0..ALL.len()).all(|i| !s.custom(i)));
        assert_eq!(s.set_len, Age::Three.set_len());
    }
}
