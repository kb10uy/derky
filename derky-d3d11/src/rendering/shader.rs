//! シェーダー関係の操作

use crate::{
    comptrize, null,
    rendering::{ComPtr, Device, HresultErrorExt},
};

use std::{ffi::c_void, fs::read, path::Path};

use anyhow::{Context, Result};
use winapi::um::d3d11;

/// Vertex Shader を保持する。
pub type VertexShader = (ComPtr<d3d11::ID3D11VertexShader>, Box<[u8]>);

/// Pixel Shader を保持する。
pub type PixelShader = ComPtr<d3d11::ID3D11PixelShader>;

/// Input Layout を表す。
pub type InputLayout = ComPtr<d3d11::ID3D11InputLayout>;

// /// Compute Shader を保持する。
// pub type ComputeShader = ComPtr<d3d11::ID3D11ComputeShader>;

/// Input Layout を作成する。
pub fn create_input_layout(
    device: &Device,
    layouts: &[d3d11::D3D11_INPUT_ELEMENT_DESC],
    vertex_shader_binary: &[u8],
) -> Result<InputLayout> {
    let input_layout = unsafe {
        let mut input_layout = null!(d3d11::ID3D11InputLayout);
        device
            .CreateInputLayout(
                layouts.as_ptr(),
                layouts.len() as u32,
                vertex_shader_binary.as_ptr() as *const c_void,
                vertex_shader_binary.len(),
                &mut input_layout as *mut *mut d3d11::ID3D11InputLayout,
            )
            .err()
            .context("Failed to create input layout")?;
        comptrize!(input_layout);
        input_layout
    };
    Ok(input_layout)
}

/// Vertex Shader バイナリを読み込む。
pub fn load_vertex_shader(
    device: &Device,
    filename: impl AsRef<Path>,
) -> Result<VertexShader> {
    let shader_binary = read(filename)?;

    let shader = unsafe {
        let mut shader = null!(d3d11::ID3D11VertexShader);
        device
            .CreateVertexShader(
                shader_binary.as_ptr() as *const c_void,
                shader_binary.len(),
                null!(d3d11::ID3D11ClassLinkage),
                &mut shader as *mut *mut d3d11::ID3D11VertexShader,
            )
            .err()
            .context("Failed to load vertex shader")?;
        comptrize!(shader);
        shader
    };
    Ok((shader, shader_binary.into_boxed_slice()))
}

/// Pixel Shader バイナリを読み込む。
pub fn load_pixel_shader(device: &Device, filename: impl AsRef<Path>) -> Result<PixelShader> {
    let shader_binary = read(filename)?;

    let shader = unsafe {
        let mut shader = null!(d3d11::ID3D11PixelShader);
        device
            .CreatePixelShader(
                shader_binary.as_ptr() as *const c_void,
                shader_binary.len(),
                null!(d3d11::ID3D11ClassLinkage),
                &mut shader as *mut *mut d3d11::ID3D11PixelShader,
            )
            .err()
            .context("Failed to load pixel shader")?;
        comptrize!(shader);
        shader
    };
    Ok(shader)
}

/*
pub fn load_compute_shader(
    device: &Device,
    filename: impl AsRef<Path>,
) -> Result<ComPtr<d3d11::ID3D11ComputeShader>> {
    let shader_binary = read(filename)?;

    let shader = unsafe {
        let mut shader = null!(d3d11::ID3D11ComputeShader);
        device
            .CreateComputeShader(
                shader_binary.as_ptr() as *const c_void,
                shader_binary.len(),
                null!(d3d11::ID3D11ClassLinkage),
                &mut shader as *mut *mut d3d11::ID3D11ComputeShader,
            )
            .err().context("Failed to load compute shader")?;
        comptrize!(shader);
        shader
    };
    Ok(shader)
}
*/
