mod model;

use model::{load_obj, ModelVertex, MODEL_VERTEX_LAYOUT};

use std::time::Duration;

use anyhow::Result;
use derky::{
    common::{
        environment::{Environment, View},
        model::Model,
        texture::Rgba,
    },
    d3d11::{
        buffer::{ConstantBuffer, IndexBuffer, VertexBuffer},
        context::{create_viewport, Context, Device, Viewport},
        shader::{InputLayout, PixelShader, VertexShader},
        texture::{DepthStencil, RenderTarget, Sampler, Texture},
        vertex::Topology,
    },
};
use ultraviolet::{Mat4, Vec2, Vec3, Vec4};

const BUFFER_VIEWPORT: Viewport = create_viewport((1280, 720));

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

    // D3D11 に対応するリソース ---------------------------------
    /// モデル用の `InputLayout`
    input_layout: InputLayout,

    /// モデル用の共通の `VertexShader`
    vs_common: VertexShader,

    /// G-Buffer 用の `PixelShader`
    ps_geometry: PixelShader,

    /// 共通の `Sampler`,
    sampler: Sampler,

    /// `ViewMatrices` の `ConstantBuffer`
    cb_view: ConstantBuffer<ViewMatrices>,

    /// モデル行列の `ConstantBuffer`
    cb_model: ConstantBuffer<Mat4>,

    /// G-Buffer
    g_buffer: Box<[RenderTarget]>,

    /// G-Buffer の `Texture`
    g_buffer_texture: Box<[Texture]>,

    /// G-Buffer の `DepthStencil`
    g_buffer_ds: DepthStencil,
}

impl Application {
    pub fn new(device: &Device) -> Result<Application> {
        let environment = Environment::new(View::new(
            Mat4::look_at_lh(
                Vec3::new(0.0, 1.0, -1.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            perspective_dx(60f32.to_radians(), 16.0 / 9.0, 0.1, 1024.0),
            Vec2::new(1280.0, 720.0),
        ));
        let model = load_obj(&device, "assets/models/Natsuki.obj")?;

        let vs_common =
            VertexShader::load_object(device, "assets/shaders/d3d11-compiled/geometry.vso")?;
        let ps_geometry =
            PixelShader::load_object(device, "assets/shaders/d3d11-compiled/geometry.pso")?;
        let input_layout = InputLayout::create(device, &MODEL_VERTEX_LAYOUT, &vs_common.binary())?;
        let sampler = Sampler::new(device)?;
        let cb_view = ConstantBuffer::new(device, &Default::default())?;
        let cb_model = ConstantBuffer::new(device, &Mat4::identity())?;
        let g_buffer: Box<_> = (0..3)
            .map(|_| RenderTarget::create::<f32, Rgba>(device, (1280, 720)))
            .collect::<Result<_>>()?;
        let g_buffer_texture: Box<_> = g_buffer
            .iter()
            .map(|rt| rt.create_texture(&device))
            .collect::<Result<_>>()?;
        let g_buffer_ds = DepthStencil::create(device, (1280, 720))?;

        Ok(Application {
            environment,
            model,
            vs_common,
            ps_geometry,
            input_layout,
            sampler,
            cb_view,
            cb_model,
            g_buffer,
            g_buffer_texture,
            g_buffer_ds,
        })
    }

    /// G-Buffer に対応する `Texture` を返す。
    pub fn g_buffer_textures(&self) -> &[Texture] {
        &self.g_buffer_texture
    }

    /// 更新処理をする。
    pub fn tick(&mut self, context: &Context, delta: Duration) {
        self.environment.tick(delta);
        self.cb_view
            .update(&context, &self.generate_view_matrices());
        self.cb_model.update(
            &context,
            &Mat4::from_rotation_y(self.environment.elapsed.as_secs_f32()),
        );
    }

    /// G-Buffer への描画をする。
    pub fn draw_geometry(&mut self, context: &Context) {
        self.g_buffer_ds.clear(&context);
        for rt in &self.g_buffer[..] {
            rt.clear(&context);
        }

        context.set_render_target(&self.g_buffer, Some(&self.g_buffer_ds));
        context.set_viewport(&BUFFER_VIEWPORT);
        context.set_shaders(&self.input_layout, &self.vs_common, &self.ps_geometry);
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
                0.0,
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
