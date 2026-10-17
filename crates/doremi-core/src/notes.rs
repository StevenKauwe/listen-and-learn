//! The note universe: solfège names, stable colors, exact equal-tempered
//! pitches, and the difficulty presets. Colors never change between rounds.

/// One note of the C-major scale, C4..B4.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    /// Solfège name shown on the button: "Do", "Re", ...
    pub name: &'static str,
    /// Stable button color (CSS name). Never randomized.
    pub color: &'static str,
    /// Exact equal-tempered frequency in Hz (A4 = 440).
    pub hz: f64,
}

/// The seven notes, in scale order. Index 0 == Do.
pub const NOTE_TABLE: [Note; 7] = [
    Note { name: "Do",  color: "red",    hz: 261.63 },
    Note { name: "Re",  color: "orange", hz: 293.66 },
    Note { name: "Mi",  color: "yellow", hz: 329.63 },
    Note { name: "Fa",  color: "green",  hz: 349.23 },
    Note { name: "Sol", color: "blue",   hz: 392.00 },
    Note { name: "La",  color: "purple", hz: 440.00 },
    Note { name: "Ti",  color: "pink",   hz: 493.88 },
];

/// Difficulty presets (spec §6). Adult-chosen; never auto-advanced silently.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    /// Do, Mi, Sol — widely separated pitches for the very young.
    Beginner,
    /// Do, Re, Mi — the default.
    Standard,
    /// Do through Sol.
    Intermediate,
    /// Do through Ti.
    Advanced,
}

impl Difficulty {
    /// Which scale indices (into NOTE_TABLE) are active for this difficulty.
    pub fn scale_indices(self) -> &'static [usize] {
        match self {
            Difficulty::Beginner => &[0, 2, 4],
            Difficulty::Standard => &[0, 1, 2],
            Difficulty::Intermediate => &[0, 1, 2, 3, 4],
            Difficulty::Advanced => &[0, 1, 2, 3, 4, 5, 6],
        }
    }
}

/// Map a button index in the active set to its scale index.
pub fn scale_index_for(difficulty: Difficulty, button: u8) -> usize {
    let indices = difficulty.scale_indices();
    debug_assert!(
        (button as usize) < indices.len(),
        "button index within active set"
    );
    indices[button as usize % indices.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitches_are_equal_tempered() {
        // A4 is the reference; C4 derives from it (2^(-9/12) * 440).
        let c4 = NOTE_TABLE[0].hz;
        assert!((c4 - 261.6256).abs() < 0.01);
        assert_eq!(NOTE_TABLE[5].hz, 440.00);
    }

    #[test]
    fn beginner_notes_are_widely_separated() {
        let b: Vec<usize> = Difficulty::Beginner.scale_indices().to_vec();
        assert_eq!(b, vec![0, 2, 4]); // Do, Mi, Sol
    }

    #[test]
    fn button_maps_to_scale_note() {
        assert_eq!(scale_index_for(Difficulty::Beginner, 1), 2); // Mi
        assert_eq!(scale_index_for(Difficulty::Advanced, 6), 6); // Ti
    }
}
