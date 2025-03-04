pub enum WindowMode {
    FullScreen,
    Borderless,
    Windowed,
}

pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

pub struct EngineConfig {
    pub window_title: String,
    pub window_mode: WindowMode,
    pub resolution: Resolution,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window_title: "".to_string(),
            window_mode: WindowMode::Windowed,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
        }
    }
}
