// src/tauri/src/new_commands/mod.rs
//
// New commands module structure (will replace commands.rs in Task 10).
// Submodules will be added in Tasks 6-9.

mod dtos;
pub mod stats;

pub use dtos::*;
pub use stats::*;

// Future submodules (Tasks 7-9):
// pub mod focus;
// pub mod categories;
// pub mod rules;
