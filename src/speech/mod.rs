//! Recognition, intent analysis, translation of speech.

pub mod builder;
pub mod recognizer;
pub mod events;
pub mod audio;

pub use builder::*;
pub use recognizer::*;
pub use events::*;
pub use audio::*;