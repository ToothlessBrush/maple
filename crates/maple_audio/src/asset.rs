use std::{path::Path, sync::Arc};

use kira::sound::static_sound::StaticSoundData;
use maple_engine::asset::{Asset, AssetLoader, FileLoader, IntoAsset, LoadErr};

pub(crate) enum AudioData {
    Static(StaticSoundData),
    Streaming(Arc<Path>),
}

/// [`Asset`] for Audio with the [`crate::nodes::AudioSource`] node
///
/// Static audio sources can be loaded with `assets.load("path/to/audio")`
/// Streaming audio sources can be added with `assets.add(StreamedAudio::new("path/to/audio"))`
/// see: [`StreamedAudio`]
pub struct Audio {
    pub(crate) data: AudioData,
}

impl Asset for Audio {
    type Loader = AudioLoader;
}

/// loader for audio sources
pub struct AudioLoader;

impl AssetLoader for AudioLoader {
    type Asset = Audio;
}

impl FileLoader for AudioLoader {
    fn load_path(
        &self,
        path: &std::path::Path,
        _library: &maple_engine::prelude::AssetLibrary,
    ) -> Result<Self::Asset, maple_engine::asset::LoadErr> {
        Ok(Audio {
            data: AudioData::Static(
                StaticSoundData::from_file(path).map_err(|err| LoadErr::Import(err.to_string()))?,
            ),
        })
    }
}

/// for converting into an audio asset and stores a refrence to a audio file to stream
pub struct StreamedAudio(Arc<Path>);

impl StreamedAudio {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self(Arc::from(path.as_ref()))
    }
}

impl IntoAsset<Audio> for StreamedAudio {
    fn into_asset(
        self,
        _loader: &<Audio as Asset>::Loader,
        _library: &maple_engine::prelude::AssetLibrary,
    ) -> Result<Audio, LoadErr> {
        if !self.0.exists() {
            return Err(LoadErr::Missing);
        }
        Ok(Audio {
            data: AudioData::Streaming(self.0),
        })
    }
}
