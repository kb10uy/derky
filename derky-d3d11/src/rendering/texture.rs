use crate::{
    comptrize, null,
    rendering::{ComPtr, HresultErrorExt},
};

use std::{ffi::c_void, fs::File, io::BufReader, mem::zeroed, path::Path};

use anyhow::{Context, Result};
use exr::prelude::rgba_image::*;
use image::{imageops::FilterType, io::Reader as ImageReader, GenericImageView};
use winapi::{
    shared::{dxgiformat, dxgitype},
    um::{d3d11, d3dcommon},
};

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
        let file = File::open(filename)?;
        let reader = ImageReader::new(BufReader::new(file)).with_guessed_format()?;
        let image = reader.decode()?;
        let (new_width, new_height) = Texture::roundup_dimensions(image.dimensions());
        let resized_image = image
            .resize_exact(new_width, new_height, FilterType::Gaussian)
            .to_rgba()
            .into_raw();

        let texture = unsafe {
            Texture::create_texture(
                device,
                resized_image.as_ptr() as *const c_void,
                1,
                (new_width, new_height),
            )?
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
        // TODO: 2^n にする
        let (_, (image, w, h)) = ImageInfo::read_pixels_from_file(
            filename,
            read_options::high(),
            |info| {
                let w = info.resolution.width();
                let h = info.resolution.height();
                let image = vec![0f32; w * h * 4];
                (image, w, h)
            },
            |(image, w, _), pos, pixel| {
                let base_index = (pos.y() * *w + pos.x()) * 4;
                let pixel_array: [f32; 4] = pixel.into();
                for i in 0..4 {
                    image[base_index + i] = pixel_array[i];
                }
            },
        )?;

        let texture = unsafe {
            Texture::create_texture(
                device,
                image.as_ptr() as *const c_void,
                4,
                (w as u32, h as u32),
            )?
        };
        let view = unsafe { Texture::create_view(device, texture.as_ptr())? };
        let sampler = unsafe { Texture::create_sampler(device)? };

        Ok(Texture {
            texture,
            view,
            sampler,
        })
    }

    fn roundup_dimensions((w, h): (u32, u32)) -> (u32, u32) {
        fn round(x: u32) -> u32 {
            if x.count_ones() == 1 {
                x
            } else {
                1 << (32 - x.leading_zeros())
            }
        };

        (round(w), round(h))
    }

    unsafe fn create_texture(
        device: &ComPtr<d3d11::ID3D11Device>,
        buffer: *const c_void,
        element_size: u32,
        (width, height): (u32, u32),
    ) -> Result<ComPtr<d3d11::ID3D11Texture2D>> {
        let desc = d3d11::D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
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
            pSysMem: buffer,
            SysMemPitch: width * element_size * 4,
            SysMemSlicePitch: width * height * element_size * 4,
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
