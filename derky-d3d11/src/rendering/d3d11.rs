//! Direct3D 11 の直接的な操作。

use crate::{
    comptrize, null,
    rendering::{
        ComPtr, ConstantBuffer, D3d11Vertex, HresultErrorExt, IndexBuffer, IndexInteger, Topology,
        VertexBuffer,
    },
};

use std::{
    ffi::c_void,
    mem::{size_of, zeroed},
};

use anyhow::{Context as AnyhowContext, Result};
use winapi::{
    shared::{dxgi, dxgiformat, dxgitype, minwindef::HINSTANCE__},
    um::{d3d11, d3dcommon},
    Interface,
};

/// Immediate Context などを保持する。
pub struct Context {
    pub(crate) immediate_context: ComPtr<d3d11::ID3D11DeviceContext>,
    pub(crate) swapchain: ComPtr<dxgi::IDXGISwapChain>,
    pub(crate) render_target_view: ComPtr<d3d11::ID3D11RenderTargetView>,
    pub(crate) depth_stencil_view: ComPtr<d3d11::ID3D11DepthStencilView>,
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

    /// 画面を消去する。
    pub fn clear(&self) {
        unsafe {
            let rtv = self.render_target_view.as_ptr();
            let dsv = self.depth_stencil_view.as_ptr();
            self.immediate_context
                .OMSetRenderTargets(1, &rtv, self.depth_stencil_view.as_ptr());
            self.immediate_context
                .ClearRenderTargetView(rtv, &[0.0, 0.0, 0.0, 1.0]);
            self.immediate_context.ClearDepthStencilView(
                dsv,
                d3d11::D3D11_CLEAR_DEPTH | d3d11::D3D11_CLEAR_STENCIL,
                1.0,
                0,
            );
        }
    }

    /// シェーダーをセットする。
    pub fn set_shaders(
        &self,
        input_layout: &ComPtr<d3d11::ID3D11InputLayout>,
        vertex: &ComPtr<d3d11::ID3D11VertexShader>,
        pixel: &ComPtr<d3d11::ID3D11PixelShader>,
    ) {
        unsafe {
            self.immediate_context
                .IASetInputLayout(input_layout.as_ptr());
            self.immediate_context.VSSetShader(
                vertex.as_ptr(),
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

    /// Vertex Shader の Constant Buffer をセットする。
    pub fn set_constant_buffer_vertex<T>(&self, slot: u32, constant_buffer: &ConstantBuffer<T>) {
        unsafe {
            self.immediate_context
                .VSSetConstantBuffers(slot, 1, &constant_buffer.buffer.as_ptr());
        }
    }

    /// ビューポートをセットする。
    pub fn set_viewport(&self, viewport: &d3d11::D3D11_VIEWPORT) {
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
    dimension: (u32, u32),
) -> Result<(ComPtr<d3d11::ID3D11Device>, Context)> {
    // Device, Swapchain, Immediate Context
    let (device, swapchain, immediate_context) = unsafe {
        let swapchain_desc = dxgi::DXGI_SWAP_CHAIN_DESC {
            BufferDesc: dxgitype::DXGI_MODE_DESC {
                Width: dimension.0,
                Height: dimension.1,
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
        .context("Failed to create device and swapchain")?;

        comptrize!(device, immediate_context, swapchain);
        (device, swapchain, immediate_context)
    };

    // Back buffer
    let render_target_view = unsafe {
        let mut render_target_view = null!(d3d11::ID3D11RenderTargetView);
        let mut back_buffer = null!(d3d11::ID3D11Texture2D);

        swapchain
            .GetBuffer(
                0,
                &d3d11::ID3D11Texture2D::uuidof(),
                &mut back_buffer as *mut *mut d3d11::ID3D11Texture2D as *mut *mut c_void,
            )
            .err()
            .context("Failed to create render target view")?;
        device
            .CreateRenderTargetView(
                back_buffer as *mut d3d11::ID3D11Resource,
                null!(d3d11::D3D11_RENDER_TARGET_VIEW_DESC),
                &mut render_target_view as *mut *mut d3d11::ID3D11RenderTargetView,
            )
            .err()
            .context("Failed to create render target view")?;

        comptrize!(back_buffer, render_target_view);
        drop(back_buffer);
        render_target_view
    };

    // Depth Stencil View
    let depth_stencil_view = unsafe {
        let desc = d3d11::D3D11_TEXTURE2D_DESC {
            Width: dimension.0,
            Height: dimension.1,
            MipLevels: 1,
            ArraySize: 1,
            Format: dxgiformat::DXGI_FORMAT_D32_FLOAT,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: d3d11::D3D11_USAGE_DEFAULT,
            BindFlags: d3d11::D3D11_BIND_DEPTH_STENCIL,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };

        let mut depth_stencil_texture = null!(d3d11::ID3D11Texture2D);
        device
            .CreateTexture2D(
                &desc,
                null!(d3d11::D3D11_SUBRESOURCE_DATA),
                &mut depth_stencil_texture as *mut *mut d3d11::ID3D11Texture2D,
            )
            .err()
            .context("Failed to create Depth Stencil Texture")?;
        comptrize!(depth_stencil_texture);

        let mut desc_ds = d3d11::D3D11_DEPTH_STENCIL_VIEW_DESC {
            Format: dxgiformat::DXGI_FORMAT_D32_FLOAT,
            ViewDimension: d3d11::D3D11_DSV_DIMENSION_TEXTURE2D,
            Flags: 0,
            u: zeroed(),
        };
        desc_ds.u.Texture2D_mut().MipSlice = 0;

        let mut depth_stencil_view = null!(d3d11::ID3D11DepthStencilView);
        device
            .CreateDepthStencilView(
                depth_stencil_texture.as_ptr() as *mut d3d11::ID3D11Resource,
                &desc_ds,
                &mut depth_stencil_view as *mut *mut d3d11::ID3D11DepthStencilView,
            )
            .err()
            .context("Failed to create Depth Stencil View")?;
        comptrize!(depth_stencil_view);
        depth_stencil_view
    };

    Ok((
        device,
        Context {
            immediate_context,
            swapchain,
            render_target_view,
            depth_stencil_view,
        },
    ))
}

/// 全面に描画する `D3D_VIEWPORT` を作成する。
pub const fn create_viewport(dimension: (u32, u32)) -> d3d11::D3D11_VIEWPORT {
    d3d11::D3D11_VIEWPORT {
        TopLeftX: 0.0,
        TopLeftY: 0.0,
        Width: dimension.0 as f32,
        Height: dimension.1 as f32,
        MinDepth: 0.0,
        MaxDepth: 0.0,
    }
}
