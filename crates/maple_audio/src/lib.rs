//! Audio support for the maple engine
//!
//! provides the [`resource::AudioManager`] as well as [`nodes::AudioSource`] and [`nodes::AudioListener`] for playing or streaming spatial audio files through
//! [`asset::Audio`] assets.

pub mod asset;
pub mod nodes;
pub mod plugin;
pub mod resource;
pub mod settings;
pub mod sound;

pub mod prelude {
    pub use crate::asset::Audio;

    pub use crate::plugin::AudioPlugin;

    pub use crate::nodes::AudioListener;
    pub use crate::nodes::AudioSource;
    pub use crate::settings::*;
    pub use crate::sound::SoundHandle;
}
