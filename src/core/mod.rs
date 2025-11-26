//! Core abstractions for diagram processing
//!
//! This module defines the fundamental traits that all diagram types must implement,
//! following the mermaid.js architecture with SOLID principles.

mod diagram;
mod database;
mod parser;
mod renderer;
mod layout;
mod detector;
mod error;

pub use diagram::*;
pub use database::*;
pub use parser::*;
pub use renderer::*;
pub use layout::*;
pub use detector::*;
pub use error::*;