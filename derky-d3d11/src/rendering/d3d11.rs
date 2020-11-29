use crate::{
    comptrize, null,
    rendering::{ComPtr, HresultErrorExt, Topology, Vertex},
};

use std::{ffi::c_void, mem::size_of};

use anyhow::{Context as AnyhowContext, Result};
use winapi::{
    shared::{dxgi, dxgiformat, dxgitype, minwindef::HINSTANCE__},
    um::{d3d11, d3dcommon},
    Interface,
};

pub struct Context {
    pub(crate) immediate_context: ComPtr<d3d11::ID3D11DeviceContext>,
    pub(crate) swapchain: ComPtr<dxgi::IDXGISwapChain>,
    pub(crate) render_target_view: ComPtr<d3d11::ID3D11RenderTargetView>,
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.immediate_context.ClearState();
        }
    }
}

impl Context {
    pub fn present(&self) {
        unsafe {
            self.swapchain.Present(0, 0);
        }
    }

    pub fn clear(&self) {
        unsafe {
            let rtv = self.render_target_view.as_ptr();
            self.immediate_context.OMSetRenderTargets(
                1,
                &rtv,
                null!(d3d11::ID3D11DepthStencilView),
            );
            self.immediate_context
                .ClearRenderTargetView(rtv, &[0.0, 1.0, 0.0, 1.0]);
        }
    }

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

    pub fn set_vertex_buffer(
        &self,
        vertex_buffer: &ComPtr<d3d11::ID3D11Buffer>,
        topology: Topology,
    ) {
        unsafe {
            self.immediate_context.IASetVertexBuffers(
                0,
                1,
                &vertex_buffer.as_ptr(),
                &(size_of::<Vertex>() as u32),
                &0,
            );
            self.immediate_context
                .IASetPrimitiveTopology(topology.to_d3d11());
        }
    }

    pub fn set_constant_buffer_vertex(
        &self,
        slot: u32,
        constant_buffer: &ComPtr<d3d11::ID3D11Buffer>,
    ) {
        unsafe {
            self.immediate_context
                .VSSetConstantBuffers(slot, 1, &constant_buffer.as_ptr());
        }
    }
}

pub fn create_d3d11(
    window_handle: *mut c_void,
    dimension: (u32, u32),
) -> Result<(ComPtr<d3d11::ID3D11Device>, Context)> {
    let flags = 0;
    let feature_levels = [d3dcommon::D3D_FEATURE_LEVEL_11_1];

    let swapchain_desc = dxgi::DXGI_SWAP_CHAIN_DESC {
        BufferDesc: dxgitype::DXGI_MODE_DESC {
            Width: dimension.0,
            Height: dimension.1,
            Format: dxgiformat::DXGI_FORMAT_R16G16B16A16_FLOAT,
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

    unsafe {
        d3d11::D3D11CreateDeviceAndSwapChain(
            null!(dxgi::IDXGIAdapter),
            d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
            null!(HINSTANCE__),
            flags,
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
    }
    comptrize!(device, immediate_context, swapchain);

    // Back buffer
    let mut back_buffer = null!(d3d11::ID3D11Texture2D);
    let mut render_target_view = null!(d3d11::ID3D11RenderTargetView);
    unsafe {
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

        comptrize!(back_buffer);
        back_buffer.Release();
    }
    comptrize!(render_target_view);

    Ok((
        device,
        Context {
            immediate_context,
            swapchain,
            render_target_view,
        },
    ))
}
