//! シーン内の情報(ライトなど)を格納する `Environment` 関連のモジュール。

use crate::rendering::load_exr_texture;
use std::time::Duration;

use anyhow::Result;
use glium::{backend::Facade, uniform, uniforms::Uniforms, Texture2d};
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

// IBL ライト
#[derive(Debug)]
pub struct ImageLight(pub Texture2d, pub f32);

impl ImageLight {
    pub fn to_uniforms<'a>(&'a self) -> impl Uniforms + 'a {
        uniform! {
            light_image_source: &self.0,
            light_image_intensity: self.1,
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
#[derive(Debug)]
pub struct Environment {
    camera_position: Vec3,
    projection_matrix: Mat4,
    elapsed_time: Duration,
    ambient_light: AmbientLight,
    image_light: ImageLight,
    directional_lights: Vec<DirectionalLight>,
    point_lights: Vec<PointLight>,
}

impl Environment {
    pub fn new(facade: &impl Facade) -> Result<Environment> {
        let ambient_light = AmbientLight(Vec3::new(0.0, 0.0, 0.0));
        let image_light = ImageLight(load_exr_texture(facade, "assets/models/background.exr")?, 0.5);
        let directional_lights = vec![];
        let point_lights = vec![
            PointLight {
                position: Vec3::new(-0.5, 0.5, 0.0),
                intensity: Vec3::new(10.0, 0.0, 0.0),
            },
            PointLight {
                position: Vec3::new(-0.5, 0.7, 0.0),
                intensity: Vec3::new(0.0, 10.0, 0.0),
            },
            PointLight {
                position: Vec3::new(0.0, 1.9, 0.0),
                intensity: Vec3::new(20.0, 20.0, 20.0),
            },
            PointLight {
                position: Vec3::new(0.0, 0.0, 1.9),
                intensity: Vec3::new(10.0, 10.0, 10.0),
            },
        ];

        Ok(Environment {
            camera_position: Vec3::new(0.0, 0.0, 0.0),
            projection_matrix: perspective_gl(60f32.to_radians(), 16.0 / 9.0, 0.1, 1024.0),
            elapsed_time: Default::default(),
            ambient_light,
            image_light,
            directional_lights,
            point_lights,
        })
    }

    pub fn tick(&mut self, delta: Duration) {
        self.elapsed_time += delta;
        let time = self.elapsed_time.as_secs_f32();

        let light1 = &mut self.point_lights[0];
        light1.position.x = (time * 2.0).cos() * 0.9;
        light1.position.y = (time * 1.7320508).sin() * 0.3 + 0.7;
        light1.position.z = (time * 2.0).sin() * 0.9;

        let light2 = &mut self.point_lights[1];
        light2.position.x = (time * -3.0).cos() * 0.2;
        light2.position.z = (time * -3.0).sin() * 0.2;

        let light3 = &mut self.point_lights[2];
        light3.intensity = if (time * 3.14).sin() > 0.0 {
            Vec3::new(10.0, 10.0, 10.0)
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        };
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

    pub fn image_light(&self) -> &ImageLight {
        &self.image_light
    }

    pub fn point_lights(&self) -> &[PointLight] {
        &self.point_lights
    }

    pub fn directional_lights(&self) -> &[DirectionalLight] {
        &self.directional_lights
    }
}
