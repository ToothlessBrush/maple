use maple_renderer::types::render_config::VsyncMode;
use winit::dpi::{PhysicalSize, Size};

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub window_title: &'static str,
    pub resolution: Option<Resolution<u32>>,
    pub vsync: VsyncMode,
    pub window_mode: WindowMode,
    pub resizeable: bool,
    pub decorated: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window_title: "Maple Window",
            resolution: None,
            vsync: VsyncMode::default(),
            window_mode: WindowMode::default(),
            resizeable: true,
            decorated: true,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum WindowMode {
    #[default]
    Windowed,
    Borderless,
    FullScreen,
}

#[derive(Debug, Clone, Copy)]
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
