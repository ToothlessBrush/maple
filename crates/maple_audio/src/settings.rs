pub use kira::Decibels;
pub use kira::Easing;
pub use kira::Panning;
pub use kira::PlaybackRate;
pub use kira::StartTime;
pub use kira::Tween;
pub use kira::Value;
pub use kira::clock::ClockId;
pub use kira::clock::ClockTime;
pub use kira::sound::EndPosition;
pub use kira::sound::PlaybackPosition;
pub use kira::sound::Region;
use kira::sound::static_sound::StaticSoundSettings;
#[cfg(not(target_arch = "wasm32"))]
use kira::sound::streaming::StreamingSoundSettings;

/// settings for playing a [`super::asset::Audio`] with either [`super::resource::AudioManager`] or
/// [`super::nodes::AudioSource`]
pub struct SoundSettings {
    /// when to start playing
    pub start_time: StartTime,
    /// where to start playing on the audio source
    pub start_position: PlaybackPosition,
    /// where to loop the audio source
    pub loop_regions: Option<Region>,
    /// whether to play the audio in reverse
    ///
    /// not supported on streaming sources
    pub reverse: bool,
    /// volume to play at
    pub volume: Value<Decibels>,
    /// rate to play the audio at
    pub playback_rate: Value<PlaybackRate>,
    /// distributes the left and right audio in stereo
    pub panning: Value<Panning>,
    /// tween the fade in of the audio
    pub fade_in_tween: Option<Tween>,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            start_time: StartTime::default(),
            start_position: PlaybackPosition::Seconds(0f64),
            loop_regions: None,
            reverse: false,
            volume: Value::Fixed(Decibels::IDENTITY),
            playback_rate: Value::Fixed(PlaybackRate(1.0)),
            panning: Value::Fixed(Panning::CENTER),
            fade_in_tween: None,
        }
    }
}

impl From<SoundSettings> for StaticSoundSettings {
    fn from(value: SoundSettings) -> Self {
        Self {
            start_time: value.start_time,
            start_position: value.start_position,
            loop_region: value.loop_regions,
            reverse: value.reverse,
            volume: value.volume,
            playback_rate: value.playback_rate,
            panning: value.panning,
            fade_in_tween: value.fade_in_tween,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<SoundSettings> for StreamingSoundSettings {
    fn from(value: SoundSettings) -> Self {
        Self {
            start_time: value.start_time,
            start_position: value.start_position,
            loop_region: value.loop_regions,
            volume: value.volume,
            playback_rate: value.playback_rate,
            panning: value.panning,
            fade_in_tween: value.fade_in_tween,
        }
    }
}
