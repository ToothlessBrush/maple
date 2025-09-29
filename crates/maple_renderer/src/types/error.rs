use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("drawing failed: {details}")]
    Draw { details: String },
    #[error("shader compilation failed: {details}")]
    ShaderCompilation { details: String },
}
