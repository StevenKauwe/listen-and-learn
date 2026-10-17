//! Do Re Mi — pure game logic for the toddler ear-training game.
//!
//! Everything testable lives here: the finite-state machine, the
//! imprecise-touch gate, the note tables and the round scheduler. The UI
//! layer (Dioxus/WASM) renders state and forwards raw pointer events; it
//! makes no decisions. Every acceptance-test row that is about *behavior*
//! is a unit test in `tests/acceptance.rs`.
//!
//! Final guiding principle, encoded: a two-year-old holds the phone
//! imperfectly. Accidental scrolling, premature guesses and double
//! activations must never interfere with play.

pub mod config;
pub mod notes;
pub mod pointer;
pub mod state;

pub use config::Config;
pub use notes::{Difficulty, NOTE_TABLE};
pub use pointer::{Decision, TouchGate};
pub use state::{Effect, Event, Game, Phase};

/// A button index in the current round's active note set (0..active_len).
pub type ButtonIdx = u8;
