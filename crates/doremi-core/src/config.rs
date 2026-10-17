//! Adult settings (spec §8). Persisted by the UI layer; the core only
//! consumes them. Defaults chosen for a two-year-old on Standard.

/// Difficulty presets live in `notes`; the rest of the knobs live here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Config {
    /// Which notes are in play. Adult-chosen only.
    pub difficulty: crate::notes::Difficulty,
    /// Note duration in seconds (0.8..=2.0; spec default 1.2).
    pub note_seconds: f32,
    /// Synthesized volume 0..=1 (the UI applies it to a gain node).
    pub volume: f32,
    /// Speak the note name on a correct answer ("Do!").
    pub spoken_names: bool,
    /// Automatically start the next round after feedback.
    pub auto_next_round: bool,
    /// Play the celebration flourish on correct answers.
    pub celebration_sounds: bool,
    /// Assist mode: after repeated misses, replay the note and gently
    /// highlight its button.
    pub assist_mode: bool,
    /// Wrong guesses before the mystery note replays itself (2..=3;
    /// 0 disables auto-replay).
    pub auto_replay_after: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            difficulty: crate::notes::Difficulty::Standard,
            note_seconds: 1.2,
            volume: 0.8,
            spoken_names: true,
            auto_next_round: true,
            celebration_sounds: true,
            assist_mode: true,
            auto_replay_after: 2,
        }
    }
}

impl Config {
    /// Clamp every knob to its legal range (settings come from storage).
    pub fn sanitized(mut self) -> Self {
        self.note_seconds = self.note_seconds.clamp(0.8, 2.0);
        self.volume = self.volume.clamp(0.0, 1.0);
        if self.auto_replay_after != 0 {
            self.auto_replay_after = self.auto_replay_after.clamp(2, 3);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_toddler_friendly() {
        let c = Config::default();
        assert_eq!(c.difficulty, crate::notes::Difficulty::Standard);
        assert_eq!(c.note_seconds, 1.2);
        assert!(c.assist_mode);
    }

    #[test]
    fn out_of_range_settings_clamp() {
        let c = Config { note_seconds: 9.0, volume: 4.0, auto_replay_after: 7, ..Config::default() };
        let c = c.sanitized();
        assert_eq!(c.note_seconds, 2.0);
        assert_eq!(c.volume, 1.0);
        assert_eq!(c.auto_replay_after, 3);
    }
}
