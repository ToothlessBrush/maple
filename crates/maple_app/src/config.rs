use maple_renderer::types::render_config::VsyncMode;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use winit::dpi::{PhysicalSize, Size};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    pub window_title: String,
    pub resolution: Option<Resolution<u32>>,
    pub vsync: VsyncMode,
    pub window_mode: WindowMode,
    pub resizeable: bool,
    pub decorated: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_title: "Maple Window".to_string(),
            resolution: None,
            vsync: VsyncMode::default(),
            window_mode: WindowMode::default(),
            resizeable: true,
            decorated: true,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WindowMode {
    #[default]
    Windowed,
    Borderless,
    FullScreen,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Resolution<P> {
    pub width: P,
    pub height: P,
}

impl Resolution<u32> {
    pub fn physical_size(&self) -> Size {
        Size::Physical(PhysicalSize {
            width: self.width,
            height: self.height,
        })
    }
}
