mod environment;
mod model;

use crate::rendering::{
    create_input_layout, create_viewport, load_pixel_shader, load_vertex_shader, ConstantBuffer,
    Context, DepthStencil, Device, IndexBuffer, InputLayout, PixelShader, RenderTarget, Texture,
    Topology, VertexBuffer, VertexShader, Viewport,
};
use model::{load_obj, ModelVertex, MODEL_VERTEX_LAYOUT};

use std::time::Duration;

use anyhow::Result;
use derky::{model::Model, texture::Rgba};
use ultraviolet::{Mat4, Vec3, Vec4};

const BUFFER_VIEWPORT: Viewport = create_viewport((1280, 720));

#[derive(Debug)]
struct Matrices {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

pub struct Application {
    // 生のリソース --------------------------------------------
    elapsed: Duration,

    /// 変換行列
    matrices: Matrices,

    /// モデル
    model: Model<(VertexBuffer<ModelVertex>, IndexBuffer<u32>), Texture>,

    // D3D11 に対応するリソース ---------------------------------
    /// モデル用の `InputLayout`
    input_layout: InputLayout,

    /// モデル用の共通の `VertexShader`
    vs_common: VertexShader,

    /// G-Buffer 用の `PixelShader`
    ps_geometry: PixelShader,

    /// `Matrices` の `ConstantBuffer`
    matrices_buffer: ConstantBuffer<Matrices>,

    /// G-Buffer
    g_buffer: Box<[RenderTarget]>,

    /// G-Buffer の `Texture`
    g_buffer_texture: Box<[Texture]>,

    /// G-Buffer の `DepthStencil`
    g_buffer_ds: DepthStencil,
}

impl Application {
    pub fn new(device: &Device) -> Result<Application> {
        let elapsed = Duration::default();
        let matrices = Matrices {
            model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            view: Mat4::look_at_lh(
                Vec3::new(0.0, 1.0, -1.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            projection: perspective_dx(60f32.to_radians(), 16.0 / 9.0, 0.1, 1024.0),
        };
        let model = load_obj(&device, "assets/models/Natsuki.obj")?;

        let vs_common = load_vertex_shader(device, "derky-d3d11/shaders/geometry.vso")?;
        let ps_geometry = load_pixel_shader(device, "derky-d3d11/shaders/geometry.pso")?;
        let input_layout = create_input_layout(device, &MODEL_VERTEX_LAYOUT, &vs_common.1)?;
        let matrices_buffer = ConstantBuffer::new(device, &matrices)?;
        let g_buffer: Box<_> = (0..2)
            .map(|_| RenderTarget::create::<f32, Rgba>(device, (1280, 720)))
            .collect::<Result<_>>()?;
        let g_buffer_texture: Box<_> = g_buffer
            .iter()
            .map(|rt| rt.create_texture(&device))
            .collect::<Result<_>>()?;
        let g_buffer_ds = DepthStencil::create(device, (1280, 720))?;

        Ok(Application {
            elapsed,
            matrices,
            model,
            vs_common,
            ps_geometry,
            input_layout,
            matrices_buffer,
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
        self.elapsed += delta;

        self.matrices.model = Mat4::from_rotation_y(self.elapsed.as_secs_f32()).into();
        self.matrices_buffer.update(&context, &self.matrices);
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
        context.set_constant_buffer_vertex(0, &self.matrices_buffer);

        for ((vb, ib), texture) in self.model.visit() {
            context.set_texture(0, texture);
            context.set_vertices(&vb, &ib, Topology::Triangles);
            context.draw_with_indices(ib.len());
        }
    }
}

/// Direct3D 用の透視投影行列を生成する。
fn perspective_dx(vertical_fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let h = 1.0 / (vertical_fov / 2.0).tan();
    let w = h / aspect;

    Mat4::new(
        Vec4::new(w, 0.0, 0.0, 0.0),
        Vec4::new(0.0, h, 0.0, 0.0),
        Vec4::new(0.0, 0.0, far / (far - near), 1.0),
        Vec4::new(0.0, 0.0, -near * far / (far - near), 0.0),
    )
}
