//! テクスチャリソース関連の型

use crate::{
    comptrize, null,
    rendering::{ComPtr, HresultErrorExt},
};

use std::{
    ffi::c_void,
    mem::{size_of, zeroed},
    path::Path,
};

use anyhow::{Context, Result};
use derky::texture::{load_hdr_image, load_ldr_image, RgbaImageData};
use winapi::{
    shared::{dxgiformat, dxgitype},
    um::{d3d11, d3dcommon},
};

/// `ID3D11Texture2D`, `ID3D11ShaderResourceView`, `ID3D11SamplerState` を保持する。
pub struct Texture {
    pub(crate) texture: ComPtr<d3d11::ID3D11Texture2D>,
    pub(crate) view: ComPtr<d3d11::ID3D11ShaderResourceView>,
    pub(crate) sampler: ComPtr<d3d11::ID3D11SamplerState>,
}

impl Texture {
    /// JPEG や PNG などの LDR テクスチャを読み込む。
    pub fn load_ldr(
        device: &ComPtr<d3d11::ID3D11Device>,
        filename: impl AsRef<Path>,
    ) -> Result<Texture> {
        let image = load_ldr_image(filename)?.resize_to_power_of_2();

        let texture = unsafe {
            Texture::create_texture(device, &image, dxgiformat::DXGI_FORMAT_R8G8B8A8_UINT)?
        };
        let view = unsafe { Texture::create_view(device, texture.as_ptr())? };
        let sampler = unsafe { Texture::create_sampler(device)? };

        Ok(Texture {
            texture,
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

        let texture = unsafe {
            Texture::create_texture(device, &image, dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT)?
        };
        let view = unsafe { Texture::create_view(device, texture.as_ptr())? };
        let sampler = unsafe { Texture::create_sampler(device)? };

        Ok(Texture {
            texture,
            view,
            sampler,
        })
    }

    /// `ID3D11Texture2D` を作成する。
    unsafe fn create_texture<T: 'static + Copy>(
        device: &ComPtr<d3d11::ID3D11Device>,
        image: &RgbaImageData<T>,
        format: dxgiformat::DXGI_FORMAT,
    ) -> Result<ComPtr<d3d11::ID3D11Texture2D>> {
        let (width, height) = image.dimensions();
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

        let channels = Texture::get_channels(format);
        let initial = d3d11::D3D11_SUBRESOURCE_DATA {
            pSysMem: image.data().as_ptr() as *const c_void,
            SysMemPitch: size_of::<T>() as u32 * width as u32 * channels,
            SysMemSlicePitch: size_of::<T>() as u32 * width as u32 * height as u32 * channels,
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

    /// `DXGI_FORMAT` からチャンネル数を判定する。
    fn get_channels(format: dxgiformat::DXGI_FORMAT) -> u32 {
        match format {
            dxgiformat::DXGI_FORMAT_R8G8_UINT
            | dxgiformat::DXGI_FORMAT_R16G16_UINT
            | dxgiformat::DXGI_FORMAT_R32G32_UINT
            | dxgiformat::DXGI_FORMAT_R16G16_FLOAT
            | dxgiformat::DXGI_FORMAT_R32G32_FLOAT => 2,
            dxgiformat::DXGI_FORMAT_R8G8B8A8_UINT
            | dxgiformat::DXGI_FORMAT_R16G16B16A16_UINT
            | dxgiformat::DXGI_FORMAT_R32G32B32A32_UINT
            | dxgiformat::DXGI_FORMAT_R16G16B16A16_FLOAT
            | dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT => 4,
            _ => todo!("Cannot judge channel count"),
        }
    }
}
