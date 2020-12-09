//! Contains types for any Buffer.

use crate::{
    comptrize,
    d3d11::{
        com_support::{ComPtr, HresultErrorExt},
        context::{Context, Device},
        vertex::D3d11Vertex,
    },
    null,
};

use std::{ffi::c_void, marker::PhantomData, mem::size_of, slice::from_ref};

use anyhow::{Context as AnyhowContext, Result};
use log::debug;
use winapi::{shared::dxgiformat, um::d3d11};

/// Inditicates that this type is available for Index Buffer.
pub trait IndexInteger {
    /// `DXGI_FORMAT` for this type
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT;
}

impl IndexInteger for u16 {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT = dxgiformat::DXGI_FORMAT_R16_UINT;
}

impl IndexInteger for u32 {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT = dxgiformat::DXGI_FORMAT_R32_UINT;
}

/// Represents a Vertex Buffer.
pub struct VertexBuffer<V: D3d11Vertex> {
    pub(crate) buffer: ComPtr<d3d11::ID3D11Buffer>,
    inner_type: PhantomData<fn() -> V>,
}

impl<V: D3d11Vertex> VertexBuffer<V> {
    /// Creates a new Vertex Buffer from vertex slice.
    pub fn new(device: &Device, vertices: &[V]) -> Result<VertexBuffer<V>> {
        let buffer = create_buffer(
            device,
            vertices,
            d3d11::D3D11_USAGE_DEFAULT,
            d3d11::D3D11_BIND_VERTEX_BUFFER,
            0,
            "Vertex",
        )?;

        Ok(VertexBuffer {
            buffer,
            inner_type: Default::default(),
        })
    }
}

/// Represents a typed Constant Buffer.
pub struct ConstantBuffer<T> {
    pub(crate) buffer: ComPtr<d3d11::ID3D11Buffer>,
    modifiable: bool,
    inner_type: PhantomData<fn() -> T>,
}

impl<T> ConstantBuffer<T> {
    /// Creates a mutable Constant Buffer.
    pub fn new(device: &Device, initial: &T) -> Result<ConstantBuffer<T>> {
        let buffer = create_buffer(
            device,
            from_ref(initial),
            d3d11::D3D11_USAGE_DEFAULT,
            d3d11::D3D11_BIND_CONSTANT_BUFFER,
            0,
            "Constant",
        )?;

        Ok(ConstantBuffer {
            buffer,
            modifiable: true,
            inner_type: Default::default(),
        })
    }

    /// Creates an immutable Constant Buffer.
    pub fn new_immutable(device: &Device, initial: &T) -> Result<ConstantBuffer<T>> {
        let buffer = create_buffer(
            device,
            from_ref(initial),
            d3d11::D3D11_USAGE_IMMUTABLE,
            d3d11::D3D11_BIND_CONSTANT_BUFFER,
            0,
            "Constant",
        )?;

        Ok(ConstantBuffer {
            buffer,
            modifiable: false,
            inner_type: Default::default(),
        })
    }

    /// Updates the data.
    pub fn update(&self, context: &Context, data: &T) {
        if !self.modifiable {
            return;
        }
        unsafe {
            context.immediate_context.UpdateSubresource(
                self.buffer.as_ptr() as *mut d3d11::ID3D11Resource,
                0,
                null!(d3d11::D3D11_BOX),
                data as *const T as *const c_void,
                0,
                0,
            );
        }
    }
}

/// Represents a Index Buffer.
pub struct IndexBuffer<T: IndexInteger> {
    pub(crate) buffer: ComPtr<d3d11::ID3D11Buffer>,
    length: usize,
    inner_type: PhantomData<fn() -> T>,
}

impl<T: IndexInteger> IndexBuffer<T> {
    /// Creates an Index Buffer from index slice.
    pub fn new(device: &Device, indices: &[T]) -> Result<IndexBuffer<T>> {
        let buffer = create_buffer(
            device,
            indices,
            d3d11::D3D11_USAGE_DEFAULT,
            d3d11::D3D11_BIND_INDEX_BUFFER,
            0,
            "Index",
        )?;

        Ok(IndexBuffer {
            buffer,
            length: indices.len(),
            inner_type: Default::default(),
        })
    }

    /// Returns the length of indices.
    pub fn len(&self) -> usize {
        self.length
    }
}

/// Creates an `ID3D11Buffer`.
fn create_buffer<T>(
    device: &Device,
    data: &[T],
    usage: d3d11::D3D11_USAGE,
    bind: d3d11::D3D11_BIND_FLAG,
    cpu_access: d3d11::D3D11_CPU_ACCESS_FLAG,
    type_string: &'static str,
) -> Result<ComPtr<d3d11::ID3D11Buffer>> {
    let byte_width = (data.len() * size_of::<T>()) as u32;
    debug!(
        "Creating {} Buffer; {} bytes, flags: ({}, {}, {})",
        type_string, byte_width, usage, bind, cpu_access
    );

    let desc = d3d11::D3D11_BUFFER_DESC {
        ByteWidth: byte_width,
        Usage: usage,
        BindFlags: bind,
        CPUAccessFlags: cpu_access,
        MiscFlags: 0,
        StructureByteStride: 0,
    };

    let initial_data = d3d11::D3D11_SUBRESOURCE_DATA {
        pSysMem: data.as_ptr() as *const c_void,
        SysMemPitch: 0,
        SysMemSlicePitch: 0,
    };

    let mut buffer = null!(d3d11::ID3D11Buffer);
    unsafe {
        device
            .device
            .CreateBuffer(
                &desc,
                &initial_data,
                &mut buffer as *mut *mut d3d11::ID3D11Buffer,
            )
            .err()
            .context(format!("Failed to create {} Buffer", type_string))?;
    }
    comptrize!(buffer);
    Ok(buffer)
}
