//! シェーダー関係の操作

use crate::{
    comptrize, null,
    rendering::{ComPtr, HresultErrorExt},
};

use std::{ffi::c_void, fs::read, path::Path};

use anyhow::{Context, Result};
use winapi::um::d3d11;

/// Vertex Shader バイナリを読み込む。
pub fn load_vertex_shader(
    device: &ComPtr<d3d11::ID3D11Device>,
    filename: impl AsRef<Path>,
) -> Result<(ComPtr<d3d11::ID3D11VertexShader>, Box<[u8]>)> {
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
pub fn load_pixel_shader(
    device: &ComPtr<d3d11::ID3D11Device>,
    filename: impl AsRef<Path>,
) -> Result<ComPtr<d3d11::ID3D11PixelShader>> {
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
    device: &ComPtr<d3d11::ID3D11Device>,
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
