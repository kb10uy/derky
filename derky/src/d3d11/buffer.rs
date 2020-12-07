// 各種バッファ操作

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
use winapi::{shared::dxgiformat, um::d3d11};

/// Index Buffer の要素に使える型が実装する trait 。
pub trait IndexInteger {
    /// DXGI_FORMAT 定数を返す。
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT;
}

impl IndexInteger for u16 {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT = dxgiformat::DXGI_FORMAT_R16_UINT;
}

impl IndexInteger for u32 {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT = dxgiformat::DXGI_FORMAT_R32_UINT;
}

/// Vertex Buffer
pub struct VertexBuffer<V: D3d11Vertex> {
    pub(crate) buffer: ComPtr<d3d11::ID3D11Buffer>,
    inner_type: PhantomData<fn() -> V>,
}

impl<V: D3d11Vertex> VertexBuffer<V> {
    /// Vertex Buffer を作成する。
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

/// 型付き Constant Buffer
pub struct ConstantBuffer<T> {
    pub(crate) buffer: ComPtr<d3d11::ID3D11Buffer>,
    modifiable: bool,
    inner_type: PhantomData<fn() -> T>,
}

impl<T> ConstantBuffer<T> {
    /// 書き換え可能な Constant Buffer を作成する。
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

    /// 内容を更新する。
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

/// 型付きの Index Buffer
pub struct IndexBuffer<T: IndexInteger> {
    pub(crate) buffer: ComPtr<d3d11::ID3D11Buffer>,
    length: usize,
    inner_type: PhantomData<fn() -> T>,
}

impl<T: IndexInteger> IndexBuffer<T> {
    /// Index Buffer を作成する。
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

    /// Index Buffer の長さを取得する。
    pub fn len(&self) -> usize {
        self.length
    }
}

/// `ID3D11Buffer` を作成する。
fn create_buffer<T>(
    device: &Device,
    data: &[T],
    usage: d3d11::D3D11_USAGE,
    bind: d3d11::D3D11_BIND_FLAG,
    cpu_access: d3d11::D3D11_CPU_ACCESS_FLAG,
    type_string: &'static str,
) -> Result<ComPtr<d3d11::ID3D11Buffer>> {
    let desc = d3d11::D3D11_BUFFER_DESC {
        ByteWidth: (data.len() * size_of::<T>()) as u32,
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
