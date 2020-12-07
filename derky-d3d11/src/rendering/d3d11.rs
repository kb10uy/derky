//! Direct3D 11 の直接的な操作。

use crate::{
    comptrize, null,
    rendering::{
        ComPtr, ConstantBuffer, D3d11Vertex, DepthStencil, HresultErrorExt, IndexBuffer,
        IndexInteger, InputLayout, PixelShader, RenderTarget, Sampler, Texture, Topology,
        VertexBuffer, VertexShader,
    },
};

use std::{ffi::c_void, mem::size_of};

use anyhow::{Context as AnyhowContext, Result};
use derky::texture::Rgba;
use winapi::{
    shared::{dxgi, dxgiformat, dxgitype, minwindef::HINSTANCE__},
    um::{d3d11, d3dcommon},
    Interface,
};

/// ビューポートを表す。
pub type Viewport = d3d11::D3D11_VIEWPORT;

/// `ID3D11Device` を保持する。
pub type Device = ComPtr<d3d11::ID3D11Device>;

/// Immediate Context などを保持する。
pub struct Context {
    pub(crate) immediate_context: ComPtr<d3d11::ID3D11DeviceContext>,
    pub(crate) swapchain: ComPtr<dxgi::IDXGISwapChain>,
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.immediate_context.ClearState();
        }
    }
}

impl Context {
    /// 画面を表示する。
    pub fn present(&self) {
        unsafe {
            self.swapchain.Present(0, 0);
        }
    }

    /// 描画対象にする `RenderTarget` と `DepthStencil` をセットする。
    pub fn set_render_target(
        &self,
        render_targets: &[RenderTarget],
        depth_stencil: Option<&DepthStencil>,
    ) {
        unsafe {
            let rtv: Vec<_> = render_targets.iter().map(|rt| rt.view.as_ptr()).collect();
            let dsv = depth_stencil.map(|ds| ds.view.as_ptr()).unwrap_or(null!(_));
            self.immediate_context
                .OMSetRenderTargets(rtv.len() as u32, rtv.as_ptr(), dsv);
        }
    }

    /// シェーダーをセットする。
    pub fn set_shaders(
        &self,
        input_layout: &InputLayout,
        vertex: &VertexShader,
        pixel: &PixelShader,
    ) {
        unsafe {
            self.immediate_context
                .IASetInputLayout(input_layout.as_ptr());
            self.immediate_context.VSSetShader(
                vertex.0.as_ptr(),
                &null!(d3d11::ID3D11ClassInstance),
                0,
            );
            self.immediate_context.PSSetShader(
                pixel.as_ptr(),
                &null!(d3d11::ID3D11ClassInstance),
                0,
            );
        }
    }

    /// Vertex Buffer, Index Buffer をセットする。
    pub fn set_vertices<V: D3d11Vertex, I: IndexInteger>(
        &self,
        vertex_buffer: &VertexBuffer<V>,
        index_buffer: &IndexBuffer<I>,
        topology: Topology,
    ) {
        unsafe {
            self.immediate_context.IASetVertexBuffers(
                0,
                1,
                &vertex_buffer.buffer.as_ptr(),
                &(size_of::<V>() as u32),
                &0,
            );
            self.immediate_context.IASetIndexBuffer(
                index_buffer.buffer.as_ptr(),
                I::DXGI_FORMAT,
                0,
            );
            self.immediate_context
                .IASetPrimitiveTopology(topology.to_d3d11());
        }
    }

    /// Texture をセットする。
    pub fn set_texture(&self, slot: usize, texture: Option<&Texture>) {
        let texture_view = texture
            .map(|p| p.view.as_ptr() as *mut d3d11::ID3D11ShaderResourceView)
            .unwrap_or(null!(_));

        unsafe {
            self.immediate_context
                .PSSetShaderResources(slot as u32, 1, &texture_view);
        }
    }

    /// Sampler をセットする。
    pub fn set_sampler(&self, slot: usize, sampler: Option<&Sampler>) {
        let sampler = sampler.map(|p| p.sampler.as_ptr()).unwrap_or(null!(_));

        unsafe {
            self.immediate_context
                .PSSetSamplers(slot as u32, 1, &sampler);
        }
    }

    /// Vertex Shader の Constant Buffer をセットする。
    pub fn set_constant_buffer_vertex<T>(&self, slot: usize, constant_buffer: &ConstantBuffer<T>) {
        unsafe {
            self.immediate_context.VSSetConstantBuffers(
                slot as u32,
                1,
                &constant_buffer.buffer.as_ptr(),
            );
        }
    }

    /// ビューポートをセットする。
    pub fn set_viewport(&self, viewport: &Viewport) {
        unsafe {
            self.immediate_context.RSSetViewports(1, viewport);
        }
    }

    /// セットされている Vertex Buffer と Index Bufferで描画する。
    pub fn draw_with_indices(&self, indices: usize) {
        unsafe {
            self.immediate_context.DrawIndexed(indices as u32, 0, 0);
        }
    }
}

/// Direct3D 11 を初期化する。
pub fn create_d3d11(
    window_handle: *mut c_void,
    dimension: (usize, usize),
) -> Result<(Device, Context, RenderTarget)> {
    // Device, Swapchain, Immediate Context
    let (device, swapchain, immediate_context) = unsafe {
        let swapchain_desc = dxgi::DXGI_SWAP_CHAIN_DESC {
            BufferDesc: dxgitype::DXGI_MODE_DESC {
                Width: dimension.0 as u32,
                Height: dimension.1 as u32,
                Format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
                RefreshRate: dxgitype::DXGI_RATIONAL {
                    Numerator: 0,
                    Denominator: 0,
                },
                Scaling: 0,
                ScanlineOrdering: 0,
            },
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 1,
            OutputWindow: window_handle as *mut winapi::shared::windef::HWND__,
            Windowed: 1,
            SwapEffect: dxgi::DXGI_SWAP_EFFECT_DISCARD,
            Flags: 0,
        };

        let mut swapchain = null!(dxgi::IDXGISwapChain);
        let mut device = null!(d3d11::ID3D11Device);
        let mut immediate_context = null!(d3d11::ID3D11DeviceContext);
        let feature_levels = [d3dcommon::D3D_FEATURE_LEVEL_11_1];
        d3d11::D3D11CreateDeviceAndSwapChain(
            null!(dxgi::IDXGIAdapter),
            d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
            null!(HINSTANCE__),
            0,
            feature_levels.as_ptr(),
            feature_levels.len() as u32,
            d3d11::D3D11_SDK_VERSION,
            &swapchain_desc as *const dxgi::DXGI_SWAP_CHAIN_DESC,
            &mut swapchain as *mut *mut dxgi::IDXGISwapChain,
            &mut device as *mut *mut d3d11::ID3D11Device,
            null!(d3dcommon::D3D_FEATURE_LEVEL),
            &mut immediate_context as *mut *mut d3d11::ID3D11DeviceContext,
        )
        .err()
        .context("Failed to create Device and Swapchain")?;

        comptrize!(device, immediate_context, swapchain);
        (device, swapchain, immediate_context)
    };

    // Back buffer
    let render_target = unsafe {
        let mut render_target_view = null!(d3d11::ID3D11RenderTargetView);
        let mut back_buffer = null!(d3d11::ID3D11Texture2D);

        swapchain
            .GetBuffer(
                0,
                &d3d11::ID3D11Texture2D::uuidof(),
                &mut back_buffer as *mut *mut d3d11::ID3D11Texture2D as *mut *mut c_void,
            )
            .err()
            .context("Failed to fetch Render Target Buffer")?;
        device
            .CreateRenderTargetView(
                back_buffer as *mut d3d11::ID3D11Resource,
                null!(d3d11::D3D11_RENDER_TARGET_VIEW_DESC),
                &mut render_target_view as *mut *mut d3d11::ID3D11RenderTargetView,
            )
            .err()
            .context("Failed to create Render Target View")?;

        comptrize!(back_buffer, render_target_view);
        RenderTarget::new::<u8, Rgba>(back_buffer, render_target_view, dimension)
    };

    Ok((
        device,
        Context {
            immediate_context,
            swapchain,
        },
        render_target,
    ))
}

/// 全面に描画する `Viewport` を作成する。
pub const fn create_viewport(dimension: (u32, u32)) -> Viewport {
    d3d11::D3D11_VIEWPORT {
        TopLeftX: 0.0,
        TopLeftY: 0.0,
        Width: dimension.0 as f32,
        Height: dimension.1 as f32,
        MinDepth: 0.0,
        MaxDepth: 1.0,
    }
}
