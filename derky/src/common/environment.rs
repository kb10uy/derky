//! Contains lights and environmental types.

use std::time::Duration;

use ultraviolet::{Mat4, Vec2, Vec3};

/// Represents an ambient light.
#[derive(Debug, Clone, PartialEq)]
pub struct AmbientLight {
    /// Light intensity
    pub intensity: Vec3,
}

/// Represents an image-based light.
#[derive(Debug)]
pub struct ImageLight<T> {
    /// Lighting texture
    pub texture: T,

    /// Light intensity multipler
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
    /// Light intensity
    pub intensity: Vec3,

    /// Light direction
    pub direction: Vec3,
}

/// Represents a point light.
#[derive(Debug, Clone, PartialEq)]
pub struct PointLight {
    /// Light intensity
    pub intensity: Vec3,

    /// Light direction
    pub position: Vec3,
}

/// Represents a view for 3D space.
#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub screen_dimensions: Vec2,
}

impl View {
    /// Creates a view from matrices.
    pub fn new(view_matrix: Mat4, projection_matrix: Mat4, screen_dimensions: Vec2) -> View {
        View {
            view_matrix,
            projection_matrix,
            screen_dimensions,
        }
    }
}

pub struct Environment<T> {
    pub ambient_light: AmbientLight,
    pub image_light: Option<ImageLight<T>>,
    pub directional_lights: Vec<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
    pub view: View,
    pub elapsed: Duration,
    pub luminance: [f32; 16],
}

impl<T> Environment<T> {
    pub fn new(view: View) -> Environment<T> {
        Environment {
            ambient_light: AmbientLight {
                intensity: Vec3::new(0.0, 0.0, 0.0),
            },
            image_light: None,
            directional_lights: vec![],
            point_lights: vec![],
            view,
            elapsed: Duration::default(),
            luminance: [0.0; 16],
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.elapsed += delta;
    }

    pub fn update_luminance(&mut self, luminance: f32) {
        let mut next_luminance = [0.0; 16];
        (&mut next_luminance[..15]).copy_from_slice(&self.luminance[1..]);
        next_luminance[15] = luminance;
        self.luminance = next_luminance;
    }
}
