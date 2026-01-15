//! provides platform specific traits like send sync for both wasm and standard

#[cfg(not(target_arch = "wasm32"))]
pub trait SendSync: Send + Sync {}

#[cfg(target_arch = "wasm32")]
pub trait SendSync {}

#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + Sync + ?Sized> SendSync for T {}

#[cfg(target_arch = "wasm32")]
impl<T: ?Sized> SendSync for T {}
