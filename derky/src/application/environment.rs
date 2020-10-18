//! シーン内の情報(ライトなど)を格納する `Environment` 関連のモジュール。

use glium::{uniform, uniforms::Uniforms};
use ultraviolet::{projection::perspective_gl, Mat4, Vec3};

/// シーンの状態を表す。
#[derive(Debug, Clone)]
pub struct Environment {
    camera_position: Vec3,
    projection_matrix: Mat4,
    directional_light: Vec3,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            camera_position: Vec3::new(0.0, 0.0, 0.0),
            projection_matrix: perspective_gl(60f32.to_radians(), 16.0 / 9.0, 0.1, 1024.0),
            directional_light: Vec3::new(0.0, -1.0, 0.0).normalized(),
        }
    }

    /// カメラ位置を設定する。
    pub fn set_camera(&mut self, position: Vec3) {
        self.camera_position = position;
    }

    /// uniforms を追加する。
    pub fn get_unforms(&self) -> impl Uniforms {
        let view: [[f32; 4]; 4] = Mat4::from_translation(-self.camera_position).into();
        let projection: [[f32; 4]; 4] = self.projection_matrix.into();
        // let directional: [f32; 3] = self.directional_light.into();
        let camera: [f32; 3] = self.camera_position.into();

        uniform! {
            env_view_matrix: view,
            env_projection_matrix: projection,
            env_camera_position: camera,
        }
    }
}
