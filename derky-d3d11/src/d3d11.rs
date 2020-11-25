use std::{ffi::c_void, ptr::NonNull, sync::Arc};

use anyhow::{bail, format_err, Result};
use log::{error, info};
use winapi::{
    shared::{
        dxgi, dxgiformat, dxgitype, minwindef,
        winerror::{HRESULT, SUCCEEDED},
    },
    um::{d3d11, d3dcommon},
    Interface,
};

/// HRESULT から Result への変換
trait HresultErrorExt {
    fn err(self) -> Result<()>;
}

impl HresultErrorExt for HRESULT {
    fn err(self) -> Result<()> {
        if SUCCEEDED(self) {
            Ok(())
        } else {
            bail!("HRESULT error value: 0x{:X}", self);
        }
    }
}

impl HresultErrorExt for minwindef::ULONG {
    fn err(self) -> Result<()> {
        if self == 0 {
            Ok(())
        } else {
            bail!("ULONG error value: 0x{:X}", self);
        }
    }
}

/// NULL 回避
macro_rules! null {
    ($t: ty) => {
        0 as *mut $t
    };
}

/// *mut T から NonNull<T> に変換する。
/// いずれかに NULL が含まれていた場合 Err でベイルアウトする。
macro_rules! nonnullize {
    ($($i:ident),* $(,)?) => { $(
        let $i = NonNull::new($i).ok_or_else(|| format_err!("{} is NULL", stringify!($i)))?;
    )* }
}

pub struct D3d11 {
    device: NonNull<d3d11::ID3D11Device>,
    immediate_context: NonNull<d3d11::ID3D11DeviceContext>,
    swapchain: NonNull<dxgi::IDXGISwapChain>,
    render_target_view: NonNull<d3d11::ID3D11RenderTargetView>,
}

impl Drop for D3d11 {
    fn drop(&mut self) {
        unsafe {
            self.immediate_context.as_ref().ClearState();
            self.render_target_view
                .as_ref()
                .Release()
                .err()
                .expect("Failed to release ID3D11RenderTargetView");
            self.swapchain
                .as_ref()
                .Release()
                .err()
                .expect("Failed to release IDXGISwapChain");
            self.immediate_context
                .as_ref()
                .Release()
                .err()
                .expect("Failed to release ID3D11DeviceContext");
            self.device
                .as_ref()
                .Release()
                .err()
                .expect("Failed to release ID3D11Device");
        }
    }
}

impl D3d11 {
    pub fn create_d3d11(window_handle: *mut c_void, dimension: (u32, u32)) -> Result<Arc<D3d11>> {
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
                null!(minwindef::HINSTANCE__),
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
            .err()?;
        }
        nonnullize!(device, immediate_context, swapchain);

        // Back buffer
        let mut back_buffer = null!(d3d11::ID3D11Texture2D);
        let mut render_target_view = null!(d3d11::ID3D11RenderTargetView);
        unsafe {
            swapchain
                .as_ref()
                .GetBuffer(
                    0,
                    &d3d11::ID3D11Texture2D::uuidof(),
                    &mut back_buffer as *mut *mut d3d11::ID3D11Texture2D as *mut *mut c_void,
                )
                .err()?;
            device
                .as_ref()
                .CreateRenderTargetView(
                    back_buffer as *mut d3d11::ID3D11Resource,
                    null!(d3d11::D3D11_RENDER_TARGET_VIEW_DESC),
                    &mut render_target_view as *mut *mut d3d11::ID3D11RenderTargetView,
                )
                .err()?;

            nonnullize!(back_buffer);
            back_buffer.as_ref().Release();
        }
        nonnullize!(render_target_view);

        Ok(Arc::new(D3d11 {
            device,
            immediate_context,
            swapchain,
            render_target_view,
        }))
    }

    pub fn present(&self) {
        unsafe {
            self.swapchain.as_ref().Present(0, 0);
        }
    }

    pub fn clear(&self) {
        unsafe {
            let ctx = self.immediate_context.as_ref();
            let rtv = self.render_target_view.as_ptr();
            ctx.OMSetRenderTargets(1, &rtv, null!(d3d11::ID3D11DepthStencilView));
            ctx.ClearRenderTargetView(rtv, &[0.0, 1.0, 0.0, 1.0]);
        }
    }
}
