//! Defines abstract data and operations for generic data,
//! and general purpose rendering framework.

/// Common operations
pub mod common {
    pub mod environment;
    pub mod model;
    pub mod texture;
}

/// Rendering framework with Direct3D 11.
pub mod d3d11 {
    pub mod buffer;
    pub mod com_support;
    pub mod context;
    pub mod shader;
    pub mod texture;
    pub mod vertex;
}
