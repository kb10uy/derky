//! シェーダー関係の操作

use crate::{
    comptrize,
    d3d11::{
        com_support::{ComPtr, HresultErrorExt},
        context::Device,
    },
    null,
};

use std::{ffi::c_void, fs::read, path::Path};

use anyhow::{Context, Result};
use winapi::um::d3d11;

/// Vertex Shader を保持する。
pub struct VertexShader {
    pub(crate) shader: ComPtr<d3d11::ID3D11VertexShader>,
    binary: Box<[u8]>,
}

impl VertexShader {
    pub fn load_object(device: &Device, filename: impl AsRef<Path>) -> Result<VertexShader> {
        let shader_binary = read(filename)?;

        let shader = unsafe {
            let mut shader = null!(d3d11::ID3D11VertexShader);
            device
                .device
                .CreateVertexShader(
                    shader_binary.as_ptr() as *const c_void,
                    shader_binary.len(),
                    null!(d3d11::ID3D11ClassLinkage),
                    &mut shader as *mut *mut d3d11::ID3D11VertexShader,
                )
                .err()
                .context("Failed to load Vertex Shader")?;
            comptrize!(shader);
            shader
        };

        Ok(VertexShader {
            shader,
            binary: shader_binary.into_boxed_slice(),
        })
    }

    pub fn binary(&self) -> &[u8] {
        &self.binary
    }
}

/// Pixel Shader を保持する。
pub struct PixelShader {
    pub(crate) shader: ComPtr<d3d11::ID3D11PixelShader>,
}

impl PixelShader {
    pub fn load_object(device: &Device, filename: impl AsRef<Path>) -> Result<PixelShader> {
        let shader_binary = read(filename)?;

        let shader = unsafe {
            let mut shader = null!(d3d11::ID3D11PixelShader);
            device
                .device
                .CreatePixelShader(
                    shader_binary.as_ptr() as *const c_void,
                    shader_binary.len(),
                    null!(d3d11::ID3D11ClassLinkage),
                    &mut shader as *mut *mut d3d11::ID3D11PixelShader,
                )
                .err()
                .context("Failed to load Pixel Shader")?;
            comptrize!(shader);
            shader
        };
        Ok(PixelShader { shader })
    }
}

/// Input Layout を表す。
pub struct InputLayout {
    pub(crate) layout: ComPtr<d3d11::ID3D11InputLayout>,
}

impl InputLayout {
    pub fn create(
        device: &Device,
        layouts: &[d3d11::D3D11_INPUT_ELEMENT_DESC],
        vertex_shader_binary: &[u8],
    ) -> Result<InputLayout> {
        let input_layout = unsafe {
            let mut input_layout = null!(d3d11::ID3D11InputLayout);
            device
                .device
                .CreateInputLayout(
                    layouts.as_ptr(),
                    layouts.len() as u32,
                    vertex_shader_binary.as_ptr() as *const c_void,
                    vertex_shader_binary.len(),
                    &mut input_layout as *mut *mut d3d11::ID3D11InputLayout,
                )
                .err()
                .context("Failed to create Input Layout")?;
            comptrize!(input_layout);
            input_layout
        };
        Ok(InputLayout {
            layout: input_layout,
        })
    }
}
