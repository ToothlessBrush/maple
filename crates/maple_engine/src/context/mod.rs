//! This module contains the game context, which contains all the necessary information for the game to run.
//!
//! This includes the root scene, the frame manager, and the input manager.

mod fps_manager;
mod game_context;
mod input_manager;

// re-exports
pub use fps_manager::FPSManager;
pub use game_context::GameContext;
pub use input_manager::InputManager;
