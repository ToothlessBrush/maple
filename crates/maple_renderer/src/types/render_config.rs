#[derive(Default, Debug, Clone, Copy)]
pub struct RenderConfig {
    pub vsync: VsyncMode,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum VsyncMode {
    #[default]
    Off,
    On,
}
