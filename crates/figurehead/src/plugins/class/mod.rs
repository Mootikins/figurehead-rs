//! Class diagram plugin
//!
//! Implements class diagram parsing and rendering.

mod database;
mod detector;
mod layout;
mod parser;
mod renderer;

pub use database::{Class, ClassDatabase, Classifier, Member, Relationship, RelationshipKind, Visibility};
pub use detector::ClassDetector;
pub use layout::{ClassLayoutAlgorithm, ClassLayoutResult, PositionedClass};
pub use parser::ClassParser;
pub use renderer::ClassRenderer;
