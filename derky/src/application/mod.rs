//! 実際のアプリケーション挙動を記述する。

mod environment;
mod material;
mod model;

use crate::{
    rendering::{load_program, UniformsSet},
    wavefront_obj::Parser,
    AnyResult,
};
use environment::Environment;
use material::Material;
use model::Model;
use std::{
    f32::consts::PI,
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};

use glium::{
    framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
    implement_vertex,
    index::PrimitiveType,
    uniform,
    uniforms::Uniforms,
    Blend, BlendingFunction, Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer,
    LinearBlendingFactor, Program, Surface, VertexBuffer,
};
use log::info;
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
    pub fn draw_geometry<UG: Fn() -> U, U: Uniforms>(
        &mut self,
        geometry_buffer: &mut MultiOutputFrameBuffer,
        generate_uniforms: UG,
    ) -> AnyResult<()> {
        let angle = self.elapsed_time.as_secs_f32() * PI;
        let model_matrix: [[f32; 4]; 4] = Mat4::from_rotation_y(angle).into();

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let program = &self.program_geometry;
        self.model.visit_groups(|vb, ib, material| {
            let albedo = match material {
                Some(Material::Diffuse { albedo, .. }) => albedo,
                _ => return Ok(()),
            };

            let uniforms = UniformsSet::new(generate_uniforms())
                .add(self.environment.get_unforms())
                .add(uniform! {
                    model_matrix: model_matrix,
                    material_albedo: albedo,
                });

            geometry_buffer.draw(vb, ib, program, &uniforms, &params)?;
            Ok(())
        })?;

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
                light_directional_direction: light_direction,
                light_directional_color: light_color,
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
        let path = path.as_ref();
        let directory = path.parent().ok_or("Invalid path")?;

        let parser = Parser::new(|filename| {
            let mut include_path = PathBuf::from(directory);
            include_path.push(filename);
            let file = File::open(include_path)?;
            Ok(file)
        });

        let obj_file = File::open(path)?;
        let obj = parser.parse(obj_file)?;

        info!(
            "Wavefront OBJ Summary: {} object(s), {} material(s)",
            obj.objects().len(),
            obj.materials().len()
        );

        Model::from_obj(display, &obj, directory)
    }
}
