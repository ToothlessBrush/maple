#[derive(Default, Debug, Clone, Copy)]
pub struct RenderConfig {
    pub vsync: VsyncMode,
    pub dimensions: [u32; 2],
}

#[derive(Default, Debug, Clone, Copy)]
pub enum VsyncMode {
    #[default]
    Off,
    On,
}
