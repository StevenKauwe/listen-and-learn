//! Listen & Learn: all game logic, headless. The web crate renders,
//! plays audio and forwards pointer events; every decision is made here.

pub mod calib;
pub mod games;
pub mod gate;
pub mod music;
pub mod rng;
pub mod round;
pub mod settings;
pub mod voice;

pub use calib::{Age, Help, Tune};
pub use games::{by_slug, Game, Knobs, Question, ALL};
pub use gate::{Decision, TouchGate};
pub use round::{Effect, Event, Phase, Round, RoundCfg};
pub use settings::Settings;
