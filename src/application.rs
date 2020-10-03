//! 実際のアプリケーション挙動を記述する。

use crate::{model::Model, wavefront_obj::WavefrontObj};
use std::{
    error::Error,
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
    time::Duration,
};

use glium::{uniform, Depth, DepthTest, Display, DrawParameters, Frame, Program, Surface};
use log::error;
use ultraviolet::{projection::perspective_gl, Mat4, Vec3};

pub struct Application {
    model: Model,
    program: Program,
    count: Duration,
}

impl Application {
    pub fn new(display: &Display) -> Result<Application, Box<dyn Error + Send + Sync>> {
        let model = Application::load_model(display, "objects/thermal-grizzly.obj")?;
        let program = Application::load_program(display, "standard")?;
        Ok(Application {
            model,
            program,
            count: Duration::new(0, 0),
        })
    }

    pub fn draw(&mut self, frame: &mut Frame, delta: Duration) {
        let fov = 60.0f32.to_radians();
        let aspect = 16.0 / 9.0;
        let rot_speed = 3.1415926f32;

        let mat_model: [[f32; 4]; 4] =
            Mat4::from_rotation_z(rot_speed * self.count.as_secs_f32()).into();
        let mat_view: [[f32; 4]; 4] = Mat4::from_translation(Vec3::new(0.0, 0.0, -2.0)).into();
        let mat_projection: [[f32; 4]; 4] = perspective_gl(fov, aspect, 0.1, 1024.0).into();
        let uniforms = uniform! {
            mat_model: mat_model,
            mat_view: mat_view,
            mat_projection: mat_projection,
        };

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        frame
            .draw(
                self.model.vertex_buffer(),
                self.model.index_buffer(),
                &self.program,
                &uniforms,
                &params,
            )
            .expect("Should be drawn");

        self.count += delta;
    }

    /// モデルを読み込む。
    fn load_model(
        display: &Display,
        path: impl AsRef<Path>,
    ) -> Result<Model, Box<dyn Error + Send + Sync>> {
        let obj_file = File::open(path.as_ref())?;
        let obj = WavefrontObj::from_reader(obj_file)?;
        let group = &obj.groups()[0];
        Model::from_group(display, group)
    }

    /// シェーダーを読み込む。
    fn load_program(
        display: &Display,
        basename: &str,
    ) -> Result<Program, Box<dyn Error + Send + Sync>> {
        let mut vertex_file = BufReader::new(File::open(format!("shaders/{}.vert", basename))?);
        let mut fragment_file = BufReader::new(File::open(format!("shaders/{}.frag", basename))?);

        let mut vertex_shader = String::with_capacity(1024);
        let mut fragment_shader = String::with_capacity(1024);

        vertex_file.read_to_string(&mut vertex_shader)?;
        fragment_file.read_to_string(&mut fragment_shader)?;

        let program = Program::from_source(display, &vertex_shader, &fragment_shader, None)
            .map_err(|e| {
                error!("Failed to compile the shader: {}", e);
                e
            })?;
        Ok(program)
    }
}
