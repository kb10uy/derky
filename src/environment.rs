//! シーン内の情報(ライトなど)を格納する `Environment` 関連のモジュール。

use glium::uniforms::{AsUniformValue, Uniforms, UniformsStorage};
use ultraviolet::{projection::perspective_gl, Mat4, Vec3};

/// シーンの状態を表す。
#[derive(Debug, Clone)]
pub struct Environment {
    view_matrix: Mat4,
    projection_matrix: Mat4,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            view_matrix: Mat4::identity(),
            projection_matrix: perspective_gl(60f32.to_radians(), 16.0 / 9.0, 0.1, 1024.0),
        }
    }

    /// カメラ位置を設定する。
    pub fn set_camera(&mut self, position: Vec3) {
        self.view_matrix = Mat4::from_translation(-position);
    }

    /// uniforms を追加する。
    pub fn add_environment(
        &self,
        source: UniformsStorage<'static, impl AsUniformValue, impl Uniforms>,
    ) -> impl Uniforms {
        let view: [[f32; 4]; 4] = self.view_matrix.into();
        let projection: [[f32; 4]; 4] = self.projection_matrix.into();
        source
            .add("mat_view", view)
            .add("mat_projection", projection)
    }
}
