//! Recognition, intent analysis, translation of speech.

pub mod audio;
pub mod builder;
pub mod events;
pub mod recognizer;

pub use audio::*;
pub use builder::*;
pub use events::*;
pub use recognizer::*;
