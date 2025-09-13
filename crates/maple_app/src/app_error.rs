use std::{error::Error, fmt::Display};

use winit::error::{EventLoopError, OsError};

#[derive(Debug)]
pub enum AppError {
    RunError(EventLoopError),
    CreateWindowError(OsError),
}

impl From<EventLoopError> for AppError {
    fn from(value: EventLoopError) -> Self {
        Self::RunError(value)
    }
}

impl From<OsError> for AppError {
    fn from(value: OsError) -> Self {
        Self::CreateWindowError(value)
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::RunError(e) => write!(f, "failed to run event loop: {e}"),
            AppError::CreateWindowError(e) => write!(f, "failed to create window: {e}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::RunError(e) => Some(e),
            AppError::CreateWindowError(e) => Some(e),
        }
    }
}
