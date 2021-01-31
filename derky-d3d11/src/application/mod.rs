mod model;

use model::{load_obj, ModelVertex, MODEL_VERTEX_LAYOUT};

use std::{collections::HashMap, slice::from_ref, time::Duration};

use anyhow::Result;
use derky::{
    common::{
        environment::{DirectionalLight, Environment, ImageLight, View},
        model::Model,
        texture::Rgba,
    },
    d3d11::{
        buffer::{ConstantBuffer, IndexBuffer, RwBuffer, VertexBuffer},
        context::{
            create_viewport, BlendOperation, BlendPair, BlendState, BlendWeight, Context, Device,
            Viewport,
        },
        shader::{InputLayout, PixelShader, VertexShader},
        texture::{DepthStencil, RenderTarget, Sampler, Texture},
        vertex::{Topology, Vertex, SCREEN_QUAD_INDICES, SCREEN_QUAD_VERTICES},
    },
};
use log::info;
use ultraviolet::{Mat4, Vec2, Vec3, Vec4};

const BUFFER_VIEWPORT: Viewport = create_viewport((1280, 720));

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum ShaderKind {
    Screen,
    Geometry,
    DirectionalLighting,
    ImageLighting,
    PointLighting,
    AmbientLighting,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum BlendStateKind {
    AlphaBlend,
    TotallyAdditional,
}

#[derive(Debug, Default)]
struct ViewMatrices {
    view: Mat4,
    projection: Mat4,
    view_inv: Mat4,
    projection_inv: Mat4,
    screen_time: Vec4,
}

pub struct Application {
    // 生のリソース --------------------------------------------
    /// 環境
    environment: Environment<Texture>,

    /// モデル
    model: Model<(VertexBuffer<ModelVertex>, IndexBuffer<u32>), Texture>,

    /// 部屋のモデル
    room_model: Model<(VertexBuffer<ModelVertex>, IndexBuffer<u32>), Texture>,

    // D3D11 に対応するリソース ---------------------------------
    /// `VertexShader` のコレクション
    vertex_shaders: HashMap<ShaderKind, VertexShader>,

    /// `PixelShader` のコレクション
    pixel_shaders: HashMap<ShaderKind, PixelShader>,

    /// `BlendState` のコレクション
    blend_states: HashMap<BlendStateKind, BlendState>,

    /// スクリーン全体の `VertexBuffer` / `IndexBuffer`
    screen_buffers: (VertexBuffer<Vertex>, IndexBuffer<u32>),

    /// モデル用の `InputLayout`
    input_layout: InputLayout,

    /// 共通の `Sampler`,
    sampler: Sampler,

    /// `ViewMatrices` の `ConstantBuffer`
    cb_view: ConstantBuffer<ViewMatrices>,

    /// モデル行列の `ConstantBuffer`
    cb_model: ConstantBuffer<Mat4>,

    /// 明度を格納する `RwBuffer`
    uav_luminance: RwBuffer<[u32; 8]>,

    /// G-Buffer
    g_buffer: Box<[RenderTarget]>,

    /// G-Buffer の `Texture`
    g_buffer_texture: Box<[Texture]>,

    /// G-Buffer の `DepthStencil`
    g_buffer_ds: DepthStencil,

    /// Lighting Buffer
    lighting_buffer: RenderTarget,

    /// Lighting Buffer の `Texture`
    lighting_buffer_texture: Texture,
}

impl Application {
    pub fn new(device: &Device) -> Result<Application> {
        let mut environment = Environment::new(View::new(
            Mat4::look_at_lh(
                Vec3::new(0.0, 1.0, -1.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            perspective_dx(60f32.to_radians(), 16.0 / 9.0, 0.1, 1024.0),
            Vec2::new(1280.0, 720.0),
        ));
        environment.image_light = Some(ImageLight {
            intensity: 1.0,
            texture: Texture::load_hdr(device, "assets/models/background.exr")?,
        });

        let model = load_obj(device, "assets/models/Natsuki.obj")?;
        let room_model = load_obj(device, "assets/models/Room.obj")?;

        let mut vertex_shaders = HashMap::new();
        let mut pixel_shaders = HashMap::new();
        let mut blend_states = HashMap::new();

        vertex_shaders.insert(
            ShaderKind::Screen,
            VertexShader::load_object(&device, "assets/shaders/d3d11-compiled/screen.vso")?,
        );
        pixel_shaders.insert(
            ShaderKind::Screen,
            PixelShader::load_object(&device, "assets/shaders/d3d11-compiled/screen.pso")?,
        );
        vertex_shaders.insert(
            ShaderKind::Geometry,
            VertexShader::load_object(device, "assets/shaders/d3d11-compiled/geometry.vso")?,
        );
        pixel_shaders.insert(
            ShaderKind::Geometry,
            PixelShader::load_object(device, "assets/shaders/d3d11-compiled/geometry.pso")?,
        );
        pixel_shaders.insert(
            ShaderKind::DirectionalLighting,
            PixelShader::load_object(
                device,
                "assets/shaders/d3d11-compiled/lighting/directional.pso",
            )?,
        );
        pixel_shaders.insert(
            ShaderKind::ImageLighting,
            PixelShader::load_object(device, "assets/shaders/d3d11-compiled/lighting/image.pso")?,
        );

        // Blend State
        blend_states.insert(
            BlendStateKind::AlphaBlend,
            BlendState::new_combined(
                device,
                (
                    BlendPair {
                        source: BlendWeight::SourceAlpha,
                        destination: BlendWeight::OneMinusSourceAlpha,
                        operation: BlendOperation::Add,
                    },
                    BlendPair {
                        source: BlendWeight::One,
                        destination: BlendWeight::Zero,
                        operation: BlendOperation::Add,
                    },
                ),
            )?,
        );
        blend_states.insert(
            BlendStateKind::TotallyAdditional,
            BlendState::new_combined(
                device,
                (
                    BlendPair {
                        source: BlendWeight::One,
                        destination: BlendWeight::One,
                        operation: BlendOperation::Add,
                    },
                    BlendPair {
                        source: BlendWeight::One,
                        destination: BlendWeight::One,
                        operation: BlendOperation::Add,
                    },
                ),
            )?,
        );

        let input_layout = InputLayout::create(
            device,
            &MODEL_VERTEX_LAYOUT,
            &vertex_shaders[&ShaderKind::Geometry].binary(),
        )?;
        let sampler = Sampler::new(device)?;

        // Buffers
        let screen_buffers = (
            VertexBuffer::new(device, &SCREEN_QUAD_VERTICES)?,
            IndexBuffer::new(device, &SCREEN_QUAD_INDICES)?,
        );
        let uav_luminance = RwBuffer::new(device, &[0u32; 8])?;
        let cb_view = ConstantBuffer::new(device, &Default::default())?;
        let cb_model = ConstantBuffer::new(device, &Mat4::identity())?;

        // G-Buffer
        let g_buffer: Box<_> = (0..3)
            .map(|_| RenderTarget::create::<f32, Rgba>(device, (1280, 720)))
            .collect::<Result<_>>()?;
        let g_buffer_texture: Box<_> = g_buffer
            .iter()
            .map(|rt| rt.create_texture(&device))
            .collect::<Result<_>>()?;
        let g_buffer_ds = DepthStencil::create(device, (1280, 720))?;

        // Lighting Buffer
        let lighting_buffer = RenderTarget::create::<f32, Rgba>(device, (1280, 720))?;
        let lighting_buffer_texture = lighting_buffer.create_texture(device)?;

        Ok(Application {
            environment,
            model,
            room_model,
            vertex_shaders,
            pixel_shaders,
            blend_states,
            input_layout,
            sampler,
            screen_buffers,
            cb_view,
            cb_model,
            uav_luminance,
            g_buffer,
            g_buffer_texture,
            g_buffer_ds,
            lighting_buffer,
            lighting_buffer_texture,
        })
    }

    /// 更新処理をする。
    pub fn tick(&mut self, context: &Context, delta: Duration) {
        self.environment.tick(delta);
        self.cb_view
            .update(&context, &self.generate_view_matrices());
        /*
        self.cb_model.update(
            &context,
            &Mat4::from_rotation_y(self.environment.elapsed.as_secs_f32()),
        );
        */
    }

    /// G-Buffer への描画をする。
    pub fn draw_geometry(&mut self, context: &Context) {
        self.g_buffer_ds.clear(&context);
        for rt in &self.g_buffer[..] {
            rt.clear(&context);
        }

        context.set_render_target(&self.g_buffer, Some(&self.g_buffer_ds));
        context.set_blend_state(
            &self.blend_states[&BlendStateKind::AlphaBlend],
            [1.0f32; 4],
            0xffffffff,
        );
        context.set_viewport(&BUFFER_VIEWPORT);
        context.set_shaders(
            &self.input_layout,
            &self.vertex_shaders[&ShaderKind::Geometry],
            &self.pixel_shaders[&ShaderKind::Geometry],
        );
        context.set_constant_buffer_vertex(0, &self.cb_view);
        context.set_constant_buffer_vertex(1, &self.cb_model);
        context.set_constant_buffer_pixel(0, &self.cb_view);
        context.set_constant_buffer_pixel(1, &self.cb_model);
        context.set_sampler(0, Some(&self.sampler));

        for ((vb, ib), texture) in self.model.visit() {
            context.set_texture(0, texture);
            context.set_vertices(&vb, &ib, Topology::Triangles);
            context.draw_with_indices(ib.len());
        }

        for ((vb, ib), texture) in self.room_model.visit() {
            context.set_texture(0, texture);
            context.set_vertices(&vb, &ib, Topology::Triangles);
            context.draw_with_indices(ib.len());
        }
    }

    /// Lighting Buffer への描画をする。
    pub fn draw_lighting(&mut self, context: &Context) {
        self.lighting_buffer.clear(&context);

        context.set_render_target(from_ref(&self.lighting_buffer), None);
        context.set_blend_state(
            &self.blend_states[&BlendStateKind::TotallyAdditional],
            [1.0f32; 4],
            0xffffffff,
        );
        context.set_viewport(&BUFFER_VIEWPORT);
        context.set_sampler(0, Some(&self.sampler));
        context.set_texture(0, Some(&self.g_buffer_texture[1]));
        context.set_texture(1, Some(&self.g_buffer_texture[2]));
        context.set_constant_buffer_vertex(0, &self.cb_view);
        context.set_constant_buffer_pixel(0, &self.cb_view);

        // Directional Lighting
        context.set_shaders(
            &self.input_layout,
            &self.vertex_shaders[&ShaderKind::Screen],
            &self.pixel_shaders[&ShaderKind::DirectionalLighting],
        );
        context.set_vertices(
            &self.screen_buffers.0,
            &self.screen_buffers.1,
            Topology::Triangles,
        );
        context.draw_with_indices(self.screen_buffers.1.len());

        // Image Lighting
        if let Some(light) = &self.environment.image_light {
            context.set_shaders(
                &self.input_layout,
                &self.vertex_shaders[&ShaderKind::Screen],
                &self.pixel_shaders[&ShaderKind::ImageLighting],
            );
            context.set_texture(2, Some(&light.texture));
            context.set_vertices(
                &self.screen_buffers.0,
                &self.screen_buffers.1,
                Topology::Triangles,
            );
            context.draw_with_indices(self.screen_buffers.1.len());
        }
    }

    /// Buffer 同士の合成をする。
    pub fn draw_composition(
        &mut self,
        context: &Context,
        target: &RenderTarget,
        depth_stencil: &DepthStencil,
    ) {
        target.clear(context);
        depth_stencil.clear(context);
        self.uav_luminance.set(context, &[0u32; 8]);

        context.set_render_targets_and_rw_buffers(
            from_ref(&target),
            Some(&depth_stencil),
            4,
            from_ref(&self.uav_luminance),
        );
        context.set_blend_state(
            &self.blend_states[&BlendStateKind::AlphaBlend],
            [1.0f32; 4],
            0xffffffff,
        );
        context.set_viewport(&BUFFER_VIEWPORT);
        context.set_sampler(0, Some(&self.sampler));
        for (index, textures) in self.g_buffer_texture.iter().enumerate() {
            context.set_texture(index, Some(textures));
        }
        context.set_texture(5, Some(&self.lighting_buffer_texture));
        context.set_shaders(
            &self.input_layout,
            &self.vertex_shaders[&ShaderKind::Screen],
            &self.pixel_shaders[&ShaderKind::Screen],
        );
        context.set_constant_buffer_pixel(0, &self.cb_view);

        context.set_vertices(
            &self.screen_buffers.0,
            &self.screen_buffers.1,
            Topology::Triangles,
        );
        context.draw_with_indices(self.screen_buffers.1.len());

        let luminance = self.uav_luminance.get(&context);
        self.environment.update_luminance(luminance[0] as f32);
        info!("Luminance: {:?}", luminance);
    }

    fn generate_view_matrices(&self) -> ViewMatrices {
        ViewMatrices {
            view: self.environment.view.view_matrix,
            projection: self.environment.view.projection_matrix,
            view_inv: self.environment.view.view_matrix.inversed(),
            projection_inv: self.environment.view.projection_matrix.inversed(),
            screen_time: Vec4::new(
                self.environment.view.screen_dimensions.x,
                self.environment.view.screen_dimensions.y,
                self.environment.elapsed.as_secs_f32(),
                self.environment.luminance,
            ),
        }
    }
}

/// Direct3D 用の透視投影行列を生成する。
fn perspective_dx(vertical_fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
    let h = 1.0 / (vertical_fov / 2.0).tan();
    let w = h / aspect_ratio;

    Mat4::new(
        Vec4::new(-w, 0.0, 0.0, 0.0),
        Vec4::new(0.0, h, 0.0, 0.0),
        Vec4::new(0.0, 0.0, far / (far - near), 1.0),
        Vec4::new(0.0, 0.0, -near * far / (far - near), 0.0),
    )
}
