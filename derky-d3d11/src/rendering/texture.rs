//! テクスチャリソース関連の型

use crate::{
    comptrize, null,
    rendering::{ComPtr, Context, HresultErrorExt},
};

use std::{
    ffi::c_void,
    mem::{size_of, zeroed},
    path::Path,
};

use anyhow::{Context as AnyhowContext, Result};
use derky::texture::{load_hdr_image, load_ldr_image, Channels, ImageData};
use winapi::{
    shared::{dxgiformat, dxgitype},
    um::{d3d11, d3dcommon},
};

/// テクスチャのピクセル要素が実装するトレイト。
pub trait TextureElement: 'static + Copy {
    /// チャンネル数に対応する `DXGI_FORMAT` を取得する。
    fn get_format(channels: usize) -> dxgiformat::DXGI_FORMAT;
}

impl TextureElement for u8 {
    fn get_format(channels: usize) -> dxgiformat::DXGI_FORMAT {
        match channels {
            1 => dxgiformat::DXGI_FORMAT_R8_UNORM,
            2 => dxgiformat::DXGI_FORMAT_R8G8_UNORM,
            4 => dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            _ => unimplemented!(),
        }
    }
}

impl TextureElement for f32 {
    fn get_format(channels: usize) -> dxgiformat::DXGI_FORMAT {
        match channels {
            1 => dxgiformat::DXGI_FORMAT_R32_FLOAT,
            2 => dxgiformat::DXGI_FORMAT_R32G32_FLOAT,
            3 => dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
            4 => dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT,
            _ => unimplemented!(),
        }
    }
}

/// `ID3D11Texture2D`, `ID3D11ShaderResourceView`, `ID3D11SamplerState` を保持する。
pub struct Texture {
    pub(crate) _texture: ComPtr<d3d11::ID3D11Texture2D>,
    pub(crate) view: ComPtr<d3d11::ID3D11ShaderResourceView>,
    pub(crate) sampler: ComPtr<d3d11::ID3D11SamplerState>,
}

impl Texture {
    pub fn new<T: TextureElement, C: Channels>(
        device: &ComPtr<d3d11::ID3D11Device>,
        data: &ImageData<T, C>,
    ) -> Result<Texture> {
        let texture = unsafe { Texture::create_texture(device, data)? };
        let view = unsafe { Texture::create_view(device, texture.as_ptr())? };
        let sampler = unsafe { Texture::create_sampler(device)? };

        Ok(Texture {
            _texture: texture,
            view,
            sampler,
        })
    }

    pub fn update<T: TextureElement, C: Channels>(
        &self,
        context: &Context,
        data: &ImageData<T, C>,
    ) {
        let (width, height) = data.dimensions();
        let channels = C::CHANNELS;
        let row = size_of::<T>() as u32 * width as u32 * channels as u32;
        let depth = size_of::<T>() as u32 * width as u32 * height as u32 * channels as u32;
        unsafe {
            context.immediate_context.UpdateSubresource(
                self._texture.as_ptr() as *mut d3d11::ID3D11Resource,
                0,
                null!(d3d11::D3D11_BOX),
                data.data().as_ptr() as *const c_void,
                row,
                depth,
            );
        }
    }

    /// JPEG や PNG などの LDR テクスチャを読み込む。
    pub fn load_ldr(
        device: &ComPtr<d3d11::ID3D11Device>,
        filename: impl AsRef<Path>,
    ) -> Result<Texture> {
        let image = load_ldr_image(filename)?.resize_to_power_of_2();

        let texture = unsafe { Texture::create_texture(device, &image)? };
        let view = unsafe { Texture::create_view(device, texture.as_ptr())? };
        let sampler = unsafe { Texture::create_sampler(device)? };

        Ok(Texture {
            _texture: texture,
            view,
            sampler,
        })
    }

    /// OpenEXR 形式の HDR テクスチャを読み込む。
    pub fn load_hdr(
        device: &ComPtr<d3d11::ID3D11Device>,
        filename: impl AsRef<Path>,
    ) -> Result<Texture> {
        let image = load_hdr_image(filename)?.resize_to_power_of_2();

        let texture = unsafe { Texture::create_texture(device, &image)? };
        let view = unsafe { Texture::create_view(device, texture.as_ptr())? };
        let sampler = unsafe { Texture::create_sampler(device)? };

        Ok(Texture {
            _texture: texture,
            view,
            sampler,
        })
    }

    /// `ID3D11Texture2D` を作成する。
    unsafe fn create_texture<T: TextureElement, C: Channels>(
        device: &ComPtr<d3d11::ID3D11Device>,
        image: &ImageData<T, C>,
    ) -> Result<ComPtr<d3d11::ID3D11Texture2D>> {
        let (width, height) = image.dimensions();
        let channels = C::CHANNELS;
        let format = T::get_format(channels);

        let desc = d3d11::D3D11_TEXTURE2D_DESC {
            Width: width as u32,
            Height: height as u32,
            MipLevels: 1,
            ArraySize: 1,
            Format: format,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: d3d11::D3D11_USAGE_DEFAULT,
            BindFlags: d3d11::D3D11_BIND_SHADER_RESOURCE,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };

        let initial = d3d11::D3D11_SUBRESOURCE_DATA {
            pSysMem: image.data().as_ptr() as *const c_void,
            SysMemPitch: size_of::<T>() as u32 * width as u32 * channels as u32,
            SysMemSlicePitch: size_of::<T>() as u32
                * width as u32
                * height as u32
                * channels as u32,
        };

        let mut texture = null!(d3d11::ID3D11Texture2D);
        device
            .CreateTexture2D(
                &desc,
                &initial,
                &mut texture as *mut *mut d3d11::ID3D11Texture2D,
            )
            .err()
            .context("Failed to create texture")?;

        comptrize!(texture);
        Ok(texture)
    }

    /// `ID3D11ShaderResourceView` を作成する。
    unsafe fn create_view(
        device: &ComPtr<d3d11::ID3D11Device>,
        texture_ptr: *mut d3d11::ID3D11Texture2D,
    ) -> Result<ComPtr<d3d11::ID3D11ShaderResourceView>> {
        let mut srv_desc = d3d11::D3D11_SHADER_RESOURCE_VIEW_DESC {
            Format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
            ViewDimension: d3dcommon::D3D11_SRV_DIMENSION_TEXTURE2D,
            u: zeroed(),
        };
        srv_desc.u.Texture2D_mut().MipLevels = 1;

        let mut view = null!(d3d11::ID3D11ShaderResourceView);
        device
            .CreateShaderResourceView(
                texture_ptr as *mut d3d11::ID3D11Resource,
                &srv_desc,
                &mut view as *mut *mut d3d11::ID3D11ShaderResourceView,
            )
            .err()
            .context("Failed to create shader resource view")?;

        comptrize!(view);
        Ok(view)
    }

    /// `ID3D11SamplerState` を作成する。
    unsafe fn create_sampler(
        device: &ComPtr<d3d11::ID3D11Device>,
    ) -> Result<ComPtr<d3d11::ID3D11SamplerState>> {
        let sampler_desc = d3d11::D3D11_SAMPLER_DESC {
            Filter: d3d11::D3D11_FILTER_COMPARISON_MIN_MAG_MIP_LINEAR,
            AddressU: d3d11::D3D11_TEXTURE_ADDRESS_WRAP,
            AddressV: d3d11::D3D11_TEXTURE_ADDRESS_WRAP,
            AddressW: d3d11::D3D11_TEXTURE_ADDRESS_WRAP,
            MipLODBias: 0.0,
            MaxAnisotropy: 1,
            ComparisonFunc: d3d11::D3D11_COMPARISON_ALWAYS,
            BorderColor: [0.0; 4],
            MinLOD: 0.0,
            MaxLOD: d3d11::D3D11_FLOAT32_MAX,
        };

        let mut sampler = null!(d3d11::ID3D11SamplerState);
        device
            .CreateSamplerState(
                &sampler_desc,
                &mut sampler as *mut *mut d3d11::ID3D11SamplerState,
            )
            .err()?;

        comptrize!(sampler);
        Ok(sampler)
    }
}

/// 最終出力を含む Render Target を表す。
pub struct RenderTarget {
    pub(crate) _texture: ComPtr<d3d11::ID3D11Texture2D>,
    pub(crate) view: ComPtr<d3d11::ID3D11RenderTargetView>,
}

impl RenderTarget {
    pub fn new(
        texture: ComPtr<d3d11::ID3D11Texture2D>,
        view: ComPtr<d3d11::ID3D11RenderTargetView>,
    ) -> RenderTarget {
        RenderTarget {
            _texture: texture,
            view,
        }
    }

    pub fn clear(&self, context: &Context) {
        unsafe {
            context
                .immediate_context
                .ClearRenderTargetView(self.view.as_ptr(), &[0.0, 0.0, 0.0, 1.0]);
        }
    }
}

pub struct DepthStencil {
    pub(crate) _texture: ComPtr<d3d11::ID3D11Texture2D>,
    pub(crate) view: ComPtr<d3d11::ID3D11DepthStencilView>,
}

impl DepthStencil {
    pub fn create(
        device: &ComPtr<d3d11::ID3D11Device>,
        dimension: (u32, u32),
    ) -> Result<DepthStencil> {
        let format = dxgiformat::DXGI_FORMAT_D24_UNORM_S8_UINT;

        let texture = unsafe {
            let desc = d3d11::D3D11_TEXTURE2D_DESC {
                Width: dimension.0,
                Height: dimension.1,
                MipLevels: 1,
                ArraySize: 1,
                Format: format,
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
            depth_stencil_texture
        };

        let view = unsafe {
            let mut desc_ds = d3d11::D3D11_DEPTH_STENCIL_VIEW_DESC {
                Format: format,
                ViewDimension: d3d11::D3D11_DSV_DIMENSION_TEXTURE2D,
                Flags: 0,
                u: zeroed(),
            };
            desc_ds.u.Texture2D_mut().MipSlice = 0;

            let mut depth_stencil_view = null!(d3d11::ID3D11DepthStencilView);
            device
                .CreateDepthStencilView(
                    texture.as_ptr() as *mut d3d11::ID3D11Resource,
                    &desc_ds,
                    &mut depth_stencil_view as *mut *mut d3d11::ID3D11DepthStencilView,
                )
                .err()
                .context("Failed to create Depth Stencil View")?;
            comptrize!(depth_stencil_view);
            depth_stencil_view
        };

        Ok(DepthStencil {
            _texture: texture,
            view,
        })
    }

    pub fn clear(&self, context: &Context) {
        unsafe {
            context.immediate_context.ClearDepthStencilView(
                self.view.as_ptr(),
                d3d11::D3D11_CLEAR_DEPTH | d3d11::D3D11_CLEAR_STENCIL,
                1.0,
                0,
            );
        }
    }
}
