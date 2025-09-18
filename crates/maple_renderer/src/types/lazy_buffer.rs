use maple_engine::utils::Debug;
use parking_lot::RwLock;

use crate::core::{Buffer, RenderContext};

/// Generic lazy buffer that can handle any CPU->GPU buffer conversion
///
/// useful when you dont have the renderer on creation
pub struct LazyBuffer<CpuData, GpuBuffer> {
    state: RwLock<BufferState<CpuData, GpuBuffer>>,
}

pub type LazyArrayBuffer<T> = LazyBuffer<Vec<T>, Buffer<[T]>>;
pub type LazyItemBuffer<T> = LazyBuffer<T, Buffer<T>>;

enum BufferState<CpuData, GpuBuffer> {
    Dirty(CpuData),
    Uploaded(GpuBuffer),
    UploadedDirty(CpuData, GpuBuffer),
    None,
}

impl<CpuData, GpuBuffer: Clone> LazyBuffer<CpuData, GpuBuffer> {
    /// Create a new lazy buffer with initial CPU data
    pub fn new(data: CpuData) -> Self {
        Self {
            state: RwLock::new(BufferState::Dirty(data)),
        }
    }

    /// Create an empty lazy buffer
    pub fn empty() -> Self {
        Self {
            state: RwLock::new(BufferState::None),
        }
    }

    /// Get buffer, creating it with the provided upload function if needed
    pub fn get_buffer<F>(&self, upload_fn: F) -> Option<GpuBuffer>
    where
        F: FnOnce(&CpuData) -> GpuBuffer,
    {
        // First, try to take a read lock to see if the buffer is already uploaded
        {
            let state = self.state.read();
            if let BufferState::Uploaded(buffer) = &*state {
                return Some(buffer.clone());
            }
        }

        // Need to upload: take a write lock
        let mut state = self.state.write();

        match &*state {
            BufferState::Uploaded(buffer) => Some(buffer.clone()), // another thread uploaded while we were waiting
            BufferState::Dirty(_) => {
                if let BufferState::Dirty(data) = std::mem::replace(&mut *state, BufferState::None)
                {
                    let buffer = upload_fn(&data);
                    *state = BufferState::Uploaded(buffer.clone());
                    Some(buffer)
                } else {
                    Debug::print_once("race condition detected");
                    None
                }
            }
            BufferState::UploadedDirty(_, buffer) => {
                println!("warning buffer out of sync");
                // TODO write to buffer
                Some(buffer.clone())
            }
            BufferState::None => None,
        }
    }

    /// Check if buffer is uploaded and ready
    pub fn is_ready(&self) -> bool {
        let state = self.state.read();
        matches!(*state, BufferState::Uploaded(_))
    }
}
