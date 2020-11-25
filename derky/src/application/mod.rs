//! 実際のアプリケーション挙動を記述する。

mod environment;
mod material;
mod model;

use crate::rendering::{
    load_program, load_screen_program, CompositionVertex, UniformsSet, SCREEN_QUAD_INDICES,
    SCREEN_QUAD_VERTICES,
};
use environment::Environment;
use material::Material;
use model::Model;
use std::{
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{format_err, Result};
use glium::{
    framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
    index::PrimitiveType,
    uniform,
    uniforms::Uniforms,
    BackfaceCullingMode, Blend, BlendingFunction, Depth, DepthTest, Display, DrawParameters, Frame,
    IndexBuffer, LinearBlendingFactor, Program, Surface, VertexBuffer,
};
use log::info;
use ultraviolet::{Mat4, Vec3};
use weavy_crab::Parser;

pub struct Application {
    environment: Environment,
    elapsed_time: Duration,
    program_geometry: Program,
    program_ambient_lighting: Program,
    program_image_lighting: Program,
    program_directional_lighting: Program,
    program_point_lighting: Program,
    program_composition: Program,
    vertices_screen: VertexBuffer<CompositionVertex>,
    indices_screen: IndexBuffer<u16>,
    model: Model,
    model_room: Model,
}

impl Application {
    pub fn new(display: &Display) -> Result<Application> {
        let model = Application::load_model(display, "objects/Natsuki.obj")?;
        let model_room = Application::load_model(display, "objects/Room.obj")?;

        let program_geometry = load_program(display, "geometry/geometry")?;
        let program_ambient_lighting = load_screen_program(display, "lighting/ambient")?;
        let program_image_lighting = load_screen_program(display, "lighting/image")?;
        let program_directional_lighting = load_screen_program(display, "lighting/directional")?;
        let program_point_lighting = load_screen_program(display, "lighting/point")?;
        let program_composition = load_screen_program(display, "composition/composition")?;

        let vertices_screen = VertexBuffer::new(display, &SCREEN_QUAD_VERTICES)?;
        let indices_screen =
            IndexBuffer::new(display, PrimitiveType::TrianglesList, &SCREEN_QUAD_INDICES)?;

        let mut environment = Environment::new(display)?;
        environment.set_camera(Vec3::new(0.0, 1.0, 1.0));

        Ok(Application {
            environment,
            elapsed_time: Duration::new(0, 0),
            program_geometry,
            program_ambient_lighting,
            program_image_lighting,
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
    ) -> Result<()> {
        let room_matrix: [[f32; 4]; 4] = Mat4::identity().into();

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
    ) -> Result<()> {
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

        // Image-Based
        let uniforms_set = UniformsSet::new(generate_uniforms())
            .add(self.environment.get_unforms())
            .add(self.environment.image_light().to_uniforms());
        lighting_buffer.draw(
            &self.vertices_screen,
            &self.indices_screen,
            &self.program_image_lighting,
            &uniforms_set,
            &params,
        )?;

        // Directional
        let directional_lights = self.environment.directional_lights();
        for directional_light in directional_lights {
            let uniforms_set = UniformsSet::new(generate_uniforms())
                .add(self.environment.get_unforms())
                .add(directional_light.to_uniforms());
            lighting_buffer.draw(
                &self.vertices_screen,
                &self.indices_screen,
                &self.program_directional_lighting,
                &uniforms_set,
                &params,
            )?;
        }

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

    pub fn draw_composition(&mut self, frame: &mut Frame, uniforms: impl Uniforms) -> Result<()> {
        let params = DrawParameters {
            ..Default::default()
        };
        frame.draw(
            &self.vertices_screen,
            &self.indices_screen,
            &self.program_composition,
            &uniforms,
            &params,
        )?;

        Ok(())
    }

    /// モデルを読み込む。
    fn load_model(display: &Display, path: impl AsRef<Path>) -> Result<Model> {
        let path = path.as_ref();
        let directory = PathBuf::from(path.parent().ok_or_else(|| format_err!("Invalid path"))?);

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
