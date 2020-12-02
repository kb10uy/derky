//! Contains lights and environmental types.

use ultraviolet::{Mat4, Vec3};

/// Represents an ambient light.
#[derive(Debug, Clone, PartialEq)]
pub struct AmbientLight {
    pub intensity: Vec3,
}

/// Represents an image-based light.
#[derive(Debug)]
pub struct ImageLight<T> {
    pub texture: T,
    pub intensity: f32,
}

impl<T: Clone> Clone for ImageLight<T> {
    fn clone(&self) -> Self {
        ImageLight {
            texture: self.texture.clone(),
            intensity: self.intensity,
        }
    }
}

/// Represents a directional light.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectionalLight {
    pub intensity: Vec3,
    pub direction: Vec3,
}

/// Represents a point light.
#[derive(Debug, Clone, PartialEq)]
pub struct PointLight {
    pub intensity: Vec3,
    pub position: Vec3,
}

/// Represents a view for 3D space.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    view_matrix: Mat4,
    projection_matrix: Mat4,
}

impl View {
    /// Creates a view from matrices.
    pub fn new(view_matrix: Mat4, projection_matrix: Mat4) -> View {
        View {
            view_matrix,
            projection_matrix,
        }
    }

    /// Sets the view matrix.
    pub fn set_view(&mut self, matrix: Mat4) {
        self.view_matrix = matrix;
    }

    /// Sets the projection matrix.
    pub fn set_projection(&mut self, matrix: Mat4) {
        self.projection_matrix = matrix;
    }

    /// Gets the view matrix.
    pub fn view_matrix(&self) -> &Mat4 {
        &self.view_matrix
    }

    /// Gets the projection matrix.
    pub fn projection_matrix(&self) -> &Mat4 {
        &self.projection_matrix
    }
}
