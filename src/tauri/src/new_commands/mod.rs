// src/tauri/src/new_commands/mod.rs
//
// New commands module structure (will replace commands.rs in Task 10).
// Submodules will be added in Tasks 6-9.

mod dtos;
pub mod categories;
pub mod focus;
pub mod stats;

pub use categories::*;
pub use dtos::*;
pub use focus::*;
pub use stats::*;

// Future submodule (Task 9):
// pub mod rules;
