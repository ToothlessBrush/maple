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
use kira::sound::streaming::StreamingSoundSettings;

pub struct SoundSettings {
    pub start_time: StartTime,
    pub start_position: PlaybackPosition,
    pub loop_regions: Option<Region>,
    pub reverse: bool,
    pub volume: Value<Decibels>,
    pub playback_rate: Value<PlaybackRate>,
    pub panning: Value<Panning>,
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
