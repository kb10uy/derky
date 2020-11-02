//! 実際のアプリケーション挙動を記述する。

mod environment;
mod material;
mod model;

use crate::{
    rendering::{load_program, load_screen_program, UniformsSet},
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
    BackfaceCullingMode, Blend, BlendingFunction, Depth, DepthTest, Display, DrawParameters, Frame,
    IndexBuffer, LinearBlendingFactor, Program, Surface, VertexBuffer,
};
use log::info;
use ultraviolet::{Mat4, Vec3};
use weavy_crab::Parser;

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
    program_ambient_lighting: Program,
    program_directional_lighting: Program,
    program_point_lighting: Program,
    program_composition: Program,
    vertices_screen: VertexBuffer<CompositionVertex>,
    indices_screen: IndexBuffer<u16>,
    model: Model,
    model_room: Model,
}

impl Application {
    pub fn new(display: &Display) -> AnyResult<Application> {
        let model = Application::load_model(display, "objects/Natsuki.obj")?;
        let model_room = Application::load_model(display, "objects/Room.obj")?;

        let program_geometry = load_program(display, "deferred_geometry")?;
        let program_ambient_lighting = load_screen_program(display, "deferred_ambient_lighting")?;
        let program_directional_lighting =
            load_screen_program(display, "deferred_directional_lighting")?;
        let program_point_lighting = load_screen_program(display, "deferred_point_lighting")?;
        let program_composition = load_screen_program(display, "deferred_composition")?;

        let vertices_screen = VertexBuffer::new(display, &SCREEN_QUAD_VERTICES)?;
        let indices_screen =
            IndexBuffer::new(display, PrimitiveType::TrianglesList, &SCREEN_QUAD_INDICES)?;

        let mut environment = Environment::new();
        environment.set_camera(Vec3::new(0.0, 1.0, 2.0));

        Ok(Application {
            environment,
            elapsed_time: Duration::new(0, 0),
            program_geometry,
            program_ambient_lighting,
            program_directional_lighting,
            program_point_lighting,
            program_composition,
            vertices_screen,
            indices_screen,
            model,
            model_room,
        })
    }

    /// 毎フレーム呼び出される。シーン内の情報を更新する。
    pub fn tick(&mut self, delta: Duration) {
        self.elapsed_time += delta;
        self.environment.tick(delta);
    }

    /// ジオメトリパスの描画をする。
    pub fn draw_geometry<UG: Fn() -> U, U: Uniforms>(
        &mut self,
        geometry_buffer: &mut MultiOutputFrameBuffer,
        generate_uniforms: UG,
    ) -> AnyResult<()> {
        let angle = self.elapsed_time.as_secs_f32() * PI;
        let room_matrix: [[f32; 4]; 4] = Mat4::identity().into();
        let model_matrix: [[f32; 4]; 4] = Mat4::from_rotation_y(angle).into();

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            backface_culling: BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };

        let program = &self.program_geometry;

        self.model_room.visit_groups(|vb, ib, material| {
            let albedo = match material {
                Some(Material::Diffuse { albedo, .. }) => albedo,
                _ => return Ok(()),
            };

            let uniforms = UniformsSet::new(generate_uniforms())
                .add(self.environment.get_unforms())
                .add(uniform! {
                    model_matrix: room_matrix,
                    material_albedo: albedo,
                });

            geometry_buffer.draw(vb, ib, program, &uniforms, &params)?;
            Ok(())
        })?;

        self.model.visit_groups(|vb, ib, material| {
            let albedo = match material {
                Some(Material::Diffuse { albedo, .. }) => albedo,
                _ => return Ok(()),
            };

            let uniforms = UniformsSet::new(generate_uniforms())
                .add(self.environment.get_unforms())
                .add(uniform! {
                    model_matrix: room_matrix,
                    material_albedo: albedo,
                });

            geometry_buffer.draw(vb, ib, program, &uniforms, &params)?;
            Ok(())
        })?;

        Ok(())
    }

    /// ライティングパスの描画をする。
    pub fn draw_lighting<UG: Fn() -> U, U: Uniforms>(
        &mut self,
        lighting_buffer: &mut SimpleFrameBuffer,
        generate_uniforms: UG,
    ) -> AnyResult<()> {
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

        // Ambient
        let uniforms_set = UniformsSet::new(generate_uniforms())
            .add(self.environment.get_unforms())
            .add(self.environment.ambient_light().to_uniforms());
        lighting_buffer.draw(
            &self.vertices_screen,
            &self.indices_screen,
            &self.program_ambient_lighting,
            &uniforms_set,
            &params,
        )?;

        // Directional
        /*
        let light_direction: [f32; 3] = Vec3::new(0.1, -0.9, -0.4).normalized().into();
        let light_color: [f32; 3] = Vec3::new(1.0, 1.0, 1.0).into();
        let uniforms_set = UniformsSet::new(generate_uniforms())
            .add(self.environment.get_unforms())
            .add(uniform! {
                light_directional_direction: light_direction,
                light_directional_color: light_color,
            });
        lighting_buffer.draw(
            &self.vertices_screen,
            &self.indices_screen,
            &self.program_directional_lighting,
            &uniforms_set,
            &params,
        )?;
        */

        // Point
        let point_lights = self.environment.point_lights();
        for point_light in point_lights {
            let uniforms_set = UniformsSet::new(generate_uniforms())
                .add(self.environment.get_unforms())
                .add(point_light.to_uniforms());
            lighting_buffer.draw(
                &self.vertices_screen,
                &self.indices_screen,
                &self.program_point_lighting,
                &uniforms_set,
                &params,
            )?;
        }

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
        let directory = PathBuf::from(path.parent().ok_or("Invalid path")?);

        let include_base = directory.clone();
        let mut parser = Parser::new(move |filename, _| {
            let mut include_path = include_base.clone();
            include_path.push(filename);
            let file = File::open(include_path)?;
            Ok(file)
        });

        let obj_file = File::open(path)?;
        let obj = parser.parse(obj_file, ())?;

        info!(
            "Wavefront OBJ Summary: {} object(s), {} material(s)",
            obj.objects().len(),
            obj.materials().len()
        );

        Model::from_obj(display, &obj, directory)
    }
}
