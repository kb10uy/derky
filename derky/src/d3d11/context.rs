//! Direct3D 11 operations.

use crate::{
    common::texture::Rgba,
    comptrize,
    d3d11::{
        buffer::{ConstantBuffer, IndexBuffer, IndexInteger, RwBuffer, VertexBuffer},
        com_support::{ComPtr, HresultErrorExt},
        shader::{ComputeShader, InputLayout, PixelShader, VertexShader},
        texture::{DepthStencil, RenderTarget, Sampler, Texture},
        vertex::{D3d11Vertex, Topology},
    },
    null,
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

/// Contains `ID3D11Device`.
pub struct Device {
    pub(crate) device: ComPtr<d3d11::ID3D11Device>,
}

/// Contains the Immediate Context and the Swapchain.
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
    /// Calls `Present()`.
    pub fn present(&self) {
        unsafe {
            self.swapchain.Present(0, 0);
        }
    }

    /// Sets `RenderTarget`s and `DepthStencil`.
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

    /// Sets `RenderTarget`s, `DepthStencil`, and `RwBuffer`s.
    pub fn set_render_targets_and_rw_buffers<T>(
        &self,
        render_targets: &[RenderTarget],
        depth_stencil: Option<&DepthStencil>,
        rw_start: usize,
        rw_buffers: &[RwBuffer<T>],
    ) {
        unsafe {
            let rtv: Vec<_> = render_targets.iter().map(|rt| rt.view.as_ptr()).collect();
            let dsv = depth_stencil.map(|ds| ds.view.as_ptr()).unwrap_or(null!(_));
            let uav: Vec<_> = rw_buffers.iter().map(|rwb| rwb.view.as_ptr()).collect();
            self.immediate_context
                .OMSetRenderTargetsAndUnorderedAccessViews(
                    rtv.len() as u32,
                    rtv.as_ptr(),
                    dsv,
                    rw_start as u32,
                    uav.len() as u32,
                    uav.as_ptr(),
                    null!(_),
                );
        }
    }

    /// Sets a `BlendState`.
    pub fn set_blend_state(&self, blend_state: &BlendState, factor: [f32; 4], mask: u32) {
        unsafe {
            self.immediate_context
                .OMSetBlendState(blend_state.blend_state.as_ptr(), &factor, mask);
        }
    }

    /// Sets Shaders and `InputLayout`.
    pub fn set_shaders(
        &self,
        input_layout: &InputLayout,
        vertex: &VertexShader,
        pixel: &PixelShader,
    ) {
        unsafe {
            self.immediate_context
                .IASetInputLayout(input_layout.layout.as_ptr());
            self.immediate_context.VSSetShader(
                vertex.shader.as_ptr(),
                &null!(d3d11::ID3D11ClassInstance),
                0,
            );
            self.immediate_context.PSSetShader(
                pixel.shader.as_ptr(),
                &null!(d3d11::ID3D11ClassInstance),
                0,
            );
        }
    }

    /// Set `VertexBuffer` and `IndexBuffer`.
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

    /// Sets or releases `Texture`.
    pub fn set_texture(&self, slot: usize, texture: Option<&Texture>) {
        let texture_view = texture
            .map(|p| p.view.as_ptr() as *mut d3d11::ID3D11ShaderResourceView)
            .unwrap_or(null!(_));

        unsafe {
            self.immediate_context
                .PSSetShaderResources(slot as u32, 1, &texture_view);
        }
    }

    /// Sets `Sampler`.
    pub fn set_sampler(&self, slot: usize, sampler: Option<&Sampler>) {
        let sampler = sampler.map(|p| p.sampler.as_ptr()).unwrap_or(null!(_));

        unsafe {
            self.immediate_context
                .PSSetSamplers(slot as u32, 1, &sampler);
        }
    }

    /// Sets `ConstantBuffer` for Vertex Shader.
    pub fn set_constant_buffer_vertex<T>(&self, slot: usize, constant_buffer: &ConstantBuffer<T>) {
        unsafe {
            self.immediate_context.VSSetConstantBuffers(
                slot as u32,
                1,
                &constant_buffer.buffer.as_ptr(),
            );
        }
    }

    /// Sets `ConstantBuffer` for Pixel Shader.
    pub fn set_constant_buffer_pixel<T>(&self, slot: usize, constant_buffer: &ConstantBuffer<T>) {
        unsafe {
            self.immediate_context.PSSetConstantBuffers(
                slot as u32,
                1,
                &constant_buffer.buffer.as_ptr(),
            );
        }
    }

    /// Sets `Viewport`.
    pub fn set_viewport(&self, viewport: &Viewport) {
        unsafe {
            self.immediate_context.RSSetViewports(1, viewport);
        }
    }

    /// Draws with set Vertex Buffer and Index Buffer.
    pub fn draw_with_indices(&self, indices: usize) {
        unsafe {
            self.immediate_context.DrawIndexed(indices as u32, 0, 0);
        }
    }

    /// Sets a `ComputeShader`.
    pub fn set_compute_shader(&self, shader: &ComputeShader) {
        unsafe {
            self.immediate_context.CSSetShader(
                shader.shader.as_ptr(),
                &null!(d3d11::ID3D11ClassInstance),
                0,
            )
        }
    }

    /// Sets `RwBuffer`s for Compute Shader.
    pub fn set_compute_rw_buffers<T>(&self, rw_start: usize, rw_buffers: &[RwBuffer<T>]) {
        let uav: Vec<_> = rw_buffers.iter().map(|rwb| rwb.view.as_ptr()).collect();
        unsafe {
            self.immediate_context.CSSetUnorderedAccessViews(
                rw_start as u32,
                rw_buffers.len() as u32,
                uav.as_ptr(),
                null!(_),
            );
        }
    }

    /// Sets or releases `Texture` for Compute Shader.
    pub fn set_compute_texture(&self, slot: usize, texture: Option<&Texture>) {
        let texture_view = texture
            .map(|p| p.view.as_ptr() as *mut d3d11::ID3D11ShaderResourceView)
            .unwrap_or(null!(_));

        unsafe {
            self.immediate_context
                .CSSetShaderResources(slot as u32, 1, &texture_view);
        }
    }

    /// Sets `ConstantBuffer` for Compute Shader.
    pub fn set_constant_buffer_compute<T>(&self, slot: usize, constant_buffer: &ConstantBuffer<T>) {
        unsafe {
            self.immediate_context.CSSetConstantBuffers(
                slot as u32,
                1,
                &constant_buffer.buffer.as_ptr(),
            );
        }
    }

    /// Dispatch calls for Compute Shader.
    pub fn dispatch_compute(&self, x: usize, y: usize, z: usize) {
        unsafe {
            self.immediate_context.Dispatch(x as u32, y as u32, z as u32);
        }
    }
}

/// Initializes and creates Direct3D 11 device.
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
        Device { device },
        Context {
            immediate_context,
            swapchain,
        },
        render_target,
    ))
}

/// Represents a viewport.
pub type Viewport = d3d11::D3D11_VIEWPORT;

/// Creates a `Viewport` that draws the whole screen.
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

/// Represents an weight for BlendState.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendWeight {
    Zero = d3d11::D3D11_BLEND_ZERO as isize,
    One = d3d11::D3D11_BLEND_ONE as isize,
    SourceColor = d3d11::D3D11_BLEND_SRC_COLOR as isize,
    OneMinusSourceColor = d3d11::D3D11_BLEND_INV_SRC_COLOR as isize,
    SourceAlpha = d3d11::D3D11_BLEND_SRC_ALPHA as isize,
    OneMinusSourceAlpha = d3d11::D3D11_BLEND_INV_SRC_ALPHA as isize,
    DestinationAlpha = d3d11::D3D11_BLEND_DEST_ALPHA as isize,
    OneMinusDestinationAlpha = d3d11::D3D11_BLEND_INV_DEST_ALPHA as isize,
    DestinationColor = d3d11::D3D11_BLEND_DEST_COLOR as isize,
    OneMinusDestinationColor = d3d11::D3D11_BLEND_INV_DEST_COLOR as isize,
    SaturatedSourceAlpha = d3d11::D3D11_BLEND_SRC_ALPHA_SAT as isize,
    BlendFactor = d3d11::D3D11_BLEND_BLEND_FACTOR as isize,
    OneMinusBlendFactor = d3d11::D3D11_BLEND_INV_BLEND_FACTOR as isize,
    Source1Color = d3d11::D3D11_BLEND_SRC1_COLOR as isize,
    OneMinusSource1Color = d3d11::D3D11_BLEND_INV_SRC1_COLOR as isize,
    Source1Alpha = d3d11::D3D11_BLEND_SRC1_ALPHA as isize,
    OneMinusSource1Alpha = d3d11::D3D11_BLEND_INV_SRC1_ALPHA as isize,
}

/// Represents a operation for BlendState.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendOperation {
    Add = d3d11::D3D11_BLEND_OP_ADD as isize,
    Subtract = d3d11::D3D11_BLEND_OP_SUBTRACT as isize,
    SubtractInverse = d3d11::D3D11_BLEND_OP_REV_SUBTRACT as isize,
    Mininum = d3d11::D3D11_BLEND_OP_MIN as isize,
    Maximum = d3d11::D3D11_BLEND_OP_MAX as isize,
}

/// An abstruction for an blend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendPair {
    pub source: BlendWeight,
    pub destination: BlendWeight,
    pub operation: BlendOperation,
}

impl Default for BlendPair {
    fn default() -> BlendPair {
        BlendPair {
            source: BlendWeight::One,
            destination: BlendWeight::Zero,
            operation: BlendOperation::Add,
        }
    }
}

/// Wraps `ID3D11BlendState`.
pub struct BlendState {
    pub(crate) blend_state: ComPtr<d3d11::ID3D11BlendState>,
}

impl BlendState {
    /// Creates a non-independent (Shared between RenderTargets) BlendState.
    pub fn new_combined(device: &Device, state: (BlendPair, BlendPair)) -> Result<BlendState> {
        let rtb_desc = d3d11::D3D11_RENDER_TARGET_BLEND_DESC {
            BlendEnable: 1,
            SrcBlend: state.0.source as u32,
            DestBlend: state.0.destination as u32,
            BlendOp: state.0.operation as u32,
            SrcBlendAlpha: state.1.source as u32,
            DestBlendAlpha: state.1.destination as u32,
            BlendOpAlpha: state.1.operation as u32,
            RenderTargetWriteMask: d3d11::D3D11_COLOR_WRITE_ENABLE_ALL as u8,
        };

        let blend_state = unsafe {
            let blend_desc = d3d11::D3D11_BLEND_DESC {
                AlphaToCoverageEnable: 0,
                IndependentBlendEnable: 0,
                RenderTarget: [
                    rtb_desc,
                    zeroed(),
                    zeroed(),
                    zeroed(),
                    zeroed(),
                    zeroed(),
                    zeroed(),
                    zeroed(),
                ],
            };

            let mut blend_state = null!(d3d11::ID3D11BlendState);
            device
                .device
                .CreateBlendState(
                    &blend_desc,
                    &mut blend_state as *mut *mut d3d11::ID3D11BlendState,
                )
                .err()
                .context("Failed to create BlendState")?;
            comptrize!(blend_state);
            blend_state
        };

        Ok(BlendState { blend_state })
    }

    /// Creates an independent BlendState.
    pub fn new_independent(
        device: &Device,
        pairs: &[(BlendPair, BlendPair)],
    ) -> Result<BlendState> {
        let rtb_descs: Vec<_> = pairs
            .iter()
            .map(|state| d3d11::D3D11_RENDER_TARGET_BLEND_DESC {
                BlendEnable: 1,
                SrcBlend: state.0.source as u32,
                DestBlend: state.0.destination as u32,
                BlendOp: state.0.operation as u32,
                SrcBlendAlpha: state.1.source as u32,
                DestBlendAlpha: state.1.destination as u32,
                BlendOpAlpha: state.1.operation as u32,
                RenderTargetWriteMask: d3d11::D3D11_COLOR_WRITE_ENABLE_ALL as u8,
            })
            .collect();

        let mut render_targets = unsafe {
            [
                zeroed(),
                zeroed(),
                zeroed(),
                zeroed(),
                zeroed(),
                zeroed(),
                zeroed(),
                zeroed(),
            ]
        };

        let rt_dest = &mut render_targets[..rtb_descs.len()];
        rt_dest.copy_from_slice(&rtb_descs);

        let blend_state = unsafe {
            let blend_desc = d3d11::D3D11_BLEND_DESC {
                AlphaToCoverageEnable: 0,
                IndependentBlendEnable: 0,
                RenderTarget: render_targets,
            };

            let mut blend_state = null!(d3d11::ID3D11BlendState);
            device
                .device
                .CreateBlendState(
                    &blend_desc,
                    &mut blend_state as *mut *mut d3d11::ID3D11BlendState,
                )
                .err()
                .context("Failed to create BlendState")?;
            comptrize!(blend_state);
            blend_state
        };
        Ok(BlendState { blend_state })
    }
}
