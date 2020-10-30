//! シーン内の情報(ライトなど)を格納する `Environment` 関連のモジュール。

use std::time::Duration;

use glium::{uniform, uniforms::Uniforms};
use ultraviolet::{projection::perspective_gl, Mat4, Vec3};

/// アンビエントライト
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmbientLight(pub Vec3);

impl AmbientLight {
    pub fn to_uniforms(&self) -> impl Uniforms {
        let intensity: [f32; 3] = self.0.into();
        uniform! {
            light_ambient_intensity: intensity,
        }
    }
}

/// ディレクショナルライト
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalLight {
    pub intensity: Vec3,
    pub direction: Vec3,
}

impl DirectionalLight {
    pub fn to_uniforms(&self) -> impl Uniforms {
        let intensity: [f32; 3] = self.intensity.into();
        let direction: [f32; 3] = self.direction.into();
        uniform! {
            light_directional_intensity: intensity,
            light_directional_direction: direction,
        }
    }
}

/// ポイントライト
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointLight {
    intensity: Vec3,
    position: Vec3,
}

impl PointLight {
    pub fn to_uniforms(&self) -> impl Uniforms {
        let intensity: [f32; 3] = self.intensity.into();
        let position: [f32; 3] = self.position.into();
        uniform! {
            light_point_intensity: intensity,
            light_point_position: position,
        }
    }
}

/// シーンの状態を表す。
#[derive(Debug, Clone)]
pub struct Environment {
    camera_position: Vec3,
    projection_matrix: Mat4,
    elapsed_time: Duration,
    ambient_light: AmbientLight,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            camera_position: Vec3::new(0.0, 0.0, 0.0),
            projection_matrix: perspective_gl(60f32.to_radians(), 16.0 / 9.0, 0.1, 1024.0),
            elapsed_time: Default::default(),
            ambient_light: AmbientLight(Vec3::new(0.05, 0.05, 0.05)),
        }
    }

    pub fn tick(&mut self, delta: Duration) {
        self.elapsed_time += delta;
    }

    /// カメラ位置を設定する。
    pub fn set_camera(&mut self, position: Vec3) {
        self.camera_position = position;
    }

    /// uniforms を追加する。
    pub fn get_unforms(&self) -> impl Uniforms {
        let view: [[f32; 4]; 4] = Mat4::from_translation(-self.camera_position).into();
        let projection: [[f32; 4]; 4] = self.projection_matrix.into();
        let camera: [f32; 3] = self.camera_position.into();

        uniform! {
            env_view_matrix: view,
            env_projection_matrix: projection,
            env_camera_position: camera,
        }
    }

    pub fn ambient_light(&self) -> AmbientLight {
        self.ambient_light
    }
}
