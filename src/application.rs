//! 実際のアプリケーション挙動を記述する。

use crate::{environment::Environment, model::Model, wavefront_obj::WavefrontObj, AnyResult};
use std::{
    error::Error,
    f32::consts::PI,
    fs::File,
    io::{prelude::*, BufReader},
    path::Path,
    time::Duration,
};

use glium::{
    framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
    implement_vertex,
    index::PrimitiveType,
    uniform,
    uniforms::{AsUniformValue, Uniforms, UniformsStorage},
    Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer, Program, Surface, VertexBuffer,
};
use log::error;
use ultraviolet::{Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
struct CompositionVertex {
    position: [f32; 4],
    uv: [f32; 2],
}
implement_vertex!(CompositionVertex, position, uv);

pub struct Application {
    environment: Environment,
    elapsed_time: Duration,
    program_geometry: Program,
    program_lighting: Program,
    program_composition: Program,
    vertices_screen: VertexBuffer<CompositionVertex>,
    indices_screen: IndexBuffer<u16>,
    model: Model,
}

impl Application {
    pub fn new(display: &Display) -> AnyResult<Application> {
        let model = Application::load_model(display, "objects/thermal-grizzly.obj")?;

        let program_geometry = Application::load_program(display, "deferred_geometry")?;
        let program_lighting = Application::load_program(display, "deferred_geometry")?;
        let program_composition = Application::load_program(display, "deferred_composition")?;

        let vertices_screen = VertexBuffer::new(
            display,
            &[
                CompositionVertex {
                    position: [-1.0, 1.0, 0.0, 1.0],
                    uv: [0.0, 0.0],
                },
                CompositionVertex {
                    position: [1.0, 1.0, 0.0, 1.0],
                    uv: [1.0, 0.0],
                },
                CompositionVertex {
                    position: [1.0, -1.0, 0.0, 1.0],
                    uv: [1.0, 1.0],
                },
                CompositionVertex {
                    position: [-1.0, -1.0, 0.0, 1.0],
                    uv: [0.0, 1.0],
                },
            ],
        )?;
        let indices_screen =
            IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0, 3, 1, 1, 3, 2])?;

        let mut environment = Environment::new();
        environment.set_camera(Vec3::new(0.0, 0.0, 2.0));

        Ok(Application {
            environment,
            elapsed_time: Duration::new(0, 0),
            program_geometry,
            program_lighting,
            program_composition,
            vertices_screen,
            indices_screen,
            model,
        })
    }

    /// 毎フレーム呼び出される。シーン内の情報を更新する。
    pub fn tick(&mut self, delta: Duration) {
        self.elapsed_time += delta;
    }

    /// ジオメトリパスの描画をする。
    pub fn draw_geometry(&mut self, geometry_buffer: &mut MultiOutputFrameBuffer) -> AnyResult<()> {
        let angle = PI * self.elapsed_time.as_secs_f32();

        let mat_model: [[f32; 4]; 4] = Mat4::from_rotation_z(angle).into();
        let app_uniforms = uniform! {
            mat_model: mat_model,
        };

        let params = DrawParameters {
            /*
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            */
            ..Default::default()
        };

        geometry_buffer.draw(
            self.model.vertex_buffer(),
            self.model.index_buffer(),
            &self.program_geometry,
            &self.environment.add_environment(app_uniforms),
            &params,
        )?;

        Ok(())
    }

    /// ライティングパスの描画をする。
    pub fn draw_lighting(&mut self, lightsing_buffer: &mut SimpleFrameBuffer) -> AnyResult<()> {
        Ok(())
    }

    pub fn draw_composition(
        &mut self,
        frame: &mut Frame,
        uniforms: UniformsStorage<impl AsUniformValue, impl Uniforms>,
    ) -> AnyResult<()> {
        frame.draw(
            &self.vertices_screen,
            &self.indices_screen,
            &self.program_composition,
            &uniforms,
            &Default::default(),
        )?;

        Ok(())
    }

    /// モデルを読み込む。
    fn load_model(display: &Display, path: impl AsRef<Path>) -> AnyResult<Model> {
        let obj_file = File::open(path.as_ref())?;
        let obj = WavefrontObj::from_reader(obj_file)?;
        let group = &obj.groups()[0];
        Model::from_group(display, group)
    }

    /// シェーダーを読み込む。
    fn load_program(display: &Display, basename: &str) -> AnyResult<Program> {
        let mut vertex_file = BufReader::new(File::open(format!("shaders/{}.vert", basename))?);
        let mut fragment_file = BufReader::new(File::open(format!("shaders/{}.frag", basename))?);

        let mut vertex_shader = String::with_capacity(1024);
        let mut fragment_shader = String::with_capacity(1024);

        vertex_file.read_to_string(&mut vertex_shader)?;
        fragment_file.read_to_string(&mut fragment_shader)?;

        let program = Program::from_source(display, &vertex_shader, &fragment_shader, None)
            .map_err(|e| {
                error!("Failed to compile the shader \"{}\": {}", basename, e);
                e
            })?;
        Ok(program)
    }
}
