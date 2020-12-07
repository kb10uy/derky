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
use model::{load_obj, ModelGroup};

use std::time::Duration;

use anyhow::Result;
use derky::common::model::Model;
use glium::{
    framebuffer::{MultiOutputFrameBuffer, SimpleFrameBuffer},
    index::PrimitiveType,
    uniform,
    uniforms::Uniforms,
    BackfaceCullingMode, Blend, BlendingFunction, Depth, DepthTest, Display, DrawParameters, Frame,
    IndexBuffer, LinearBlendingFactor, Program, Surface, VertexBuffer,
};
use ultraviolet::{Mat4, Vec3};

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
    model: Model<ModelGroup, Material>,
    model_room: Model<ModelGroup, Material>,
}

impl Application {
    pub fn new(display: &Display) -> Result<Application> {
        let model = load_obj(display, "assets/models/Natsuki.obj")?;
        let model_room = load_obj(display, "assets/models/Room.obj")?;

        let program_geometry = load_program(display, "assets/shaders/gl4/geometry/geometry")?;
        let program_ambient_lighting =
            load_screen_program(display, "assets/shaders/gl4/lighting/ambient")?;
        let program_image_lighting =
            load_screen_program(display, "assets/shaders/gl4/lighting/image")?;
        let program_directional_lighting =
            load_screen_program(display, "assets/shaders/gl4/lighting/directional")?;
        let program_point_lighting =
            load_screen_program(display, "assets/shaders/gl4/lighting/point")?;
        let program_composition =
            load_screen_program(display, "assets/shaders/gl4/composition/composition")?;

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
        let models = [&self.model_room, &self.model];

        for &target in &models {
            for (mg, mat) in target.visit() {
                let albedo = match mat {
                    Some(Material::Diffuse { albedo, .. }) => albedo,
                    _ => return Ok(()),
                };

                let uniforms = UniformsSet::new(generate_uniforms())
                    .add(self.environment.get_unforms())
                    .add(uniform! {
                        model_matrix: room_matrix,
                        material_albedo: albedo,
                    });

                geometry_buffer.draw(
                    &mg.vertex_buffer,
                    &mg.index_buffer,
                    program,
                    &uniforms,
                    &params,
                )?;
            }
        }

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
}
