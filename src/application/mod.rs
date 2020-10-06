//! 実際のアプリケーション挙動を記述する。

mod environment;
mod material;
mod model;

use environment::Environment;
use model::Model;

use crate::{
    rendering::{load_program, UniformsSet},
    wavefront_obj::WavefrontObj,
    AnyResult,
};
use std::{f32::consts::PI, fs::File, path::Path, time::Duration};

use glium::{
    framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
    implement_vertex,
    index::PrimitiveType,
    uniform,
    uniforms::Uniforms,
    Blend, BlendingFunction, Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer,
    LinearBlendingFactor, Program, Surface, VertexBuffer,
};
use ultraviolet::{Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
struct CompositionVertex {
    position: [f32; 4],
    uv: [f32; 2],
}
implement_vertex!(CompositionVertex, position, uv);

const SCREEN_QUAD_VERTICES: [CompositionVertex; 4] = [
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
];

const SCREEN_QUAD_INDICES: [u16; 6] = [0, 3, 1, 1, 3, 2];

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
        let model = Application::load_model(display, "objects/Natsuki.obj")?;

        let program_geometry = load_program(display, "deferred_geometry")?;
        let program_lighting = load_program(display, "deferred_lighting")?;
        let program_composition = load_program(display, "deferred_composition")?;

        let vertices_screen = VertexBuffer::new(display, &SCREEN_QUAD_VERTICES)?;
        let indices_screen =
            IndexBuffer::new(display, PrimitiveType::TrianglesList, &SCREEN_QUAD_INDICES)?;

        let mut environment = Environment::new();
        environment.set_camera(Vec3::new(0.0, 1.0, 2.0));

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
    pub fn draw_geometry(
        &mut self,
        geometry_buffer: &mut MultiOutputFrameBuffer,
        uniforms: impl Uniforms,
    ) -> AnyResult<()> {
        let model_matrix: [[f32; 4]; 4] = Mat4::identity().into();
        let uniforms = UniformsSet::new(uniforms)
            .add(self.environment.get_unforms())
            .add(uniform! {
                mat_model: model_matrix,
            });

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        geometry_buffer.draw(
            self.model.vertex_buffer(),
            self.model.index_buffer(),
            &self.program_geometry,
            &uniforms,
            &params,
        )?;

        Ok(())
    }

    /// ライティングパスの描画をする。
    pub fn draw_lighting(
        &mut self,
        lighting_buffer: &mut SimpleFrameBuffer,
        uniforms: impl Uniforms,
    ) -> AnyResult<()> {
        let light_direction: [f32; 3] = Vec3::new(0.1, -0.9, -0.4).normalized().into();
        let light_color: [f32; 3] = Vec3::new(1.0, 1.0, 1.0).into();

        let uniforms = UniformsSet::new(uniforms)
            .add(self.environment.get_unforms())
            .add(uniform! {
                lit_dir_direction: light_direction,
                lit_dir_color: light_color,
            });

        let params = DrawParameters {
            blend: Blend {
                color: BlendingFunction::Addition {
                    source: LinearBlendingFactor::One,
                    destination: LinearBlendingFactor::One,
                },
                alpha: BlendingFunction::Addition {
                    source: LinearBlendingFactor::One,
                    destination: LinearBlendingFactor::One,
                },
                constant_value: (1.0, 1.0, 1.0, 1.0),
            },
            ..Default::default()
        };

        lighting_buffer.draw(
            &self.vertices_screen,
            &self.indices_screen,
            &self.program_lighting,
            &uniforms,
            &params,
        )?;

        Ok(())
    }

    pub fn draw_composition(
        &mut self,
        frame: &mut Frame,
        uniforms: impl Uniforms,
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

        Model::from_obj(display, &obj)
    }
}
