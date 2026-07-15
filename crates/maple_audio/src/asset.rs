use std::{path::PathBuf, sync::Arc};

use kira::sound::{static_sound::StaticSoundData, streaming::StreamingSoundSettings};
use maple_engine::asset::{Asset, AssetLoader, FileLoader, LoadErr};

pub(crate) enum AudioData {
    Static(StaticSoundData),
    Streaming {
        path: Arc<PathBuf>,
        settings: StreamingSoundSettings,
    },
}

pub struct Audio {
    pub(crate) data: AudioData,
}

impl Asset for Audio {
    type Loader = AudioLoader;
}

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
