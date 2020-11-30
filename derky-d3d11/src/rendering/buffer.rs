use crate::{
    comptrize, null,
    rendering::{ComPtr, Context, HresultErrorExt},
};

use std::{ffi::c_void, marker::PhantomData, mem::size_of, ops::Deref, ptr::NonNull, sync::Arc};

use anyhow::{bail, format_err, Context as AnyhowContext, Result};
use ultraviolet::{Vec2, Vec3};
use winapi::{
    shared::{
        dxgi, dxgiformat, dxgitype, minwindef,
        winerror::{HRESULT, SUCCEEDED},
    },
    um::{d3d11, d3dcommon, unknwnbase::IUnknown},
    Interface,
};

/// Vertex Shader 入力のトポロジー
#[derive(Debug, Clone, Copy)]
pub enum Topology {
    Points,
    Lines,
    LinesStrip,
    Triangles,
    TrianglesStrip,
}

impl Topology {
    pub fn to_d3d11(self) -> u32 {
        match self {
            Topology::Points => d3dcommon::D3D11_PRIMITIVE_TOPOLOGY_POINTLIST,
            Topology::Lines => d3dcommon::D3D11_PRIMITIVE_TOPOLOGY_LINELIST,
            Topology::LinesStrip => d3dcommon::D3D11_PRIMITIVE_TOPOLOGY_LINESTRIP,
            Topology::Triangles => d3dcommon::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            Topology::TrianglesStrip => d3dcommon::D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
        }
    }
}

/// 頂点
/// TODO: アラインメント調整が必要？
#[derive(Debug, Clone)]
pub struct Vertex {
    position: Vec3,
    normal: Vec3,
    uv: Vec2,
}

/// `Vertex` の InputLayout
pub const VERTEX_LAYOUT: [d3d11::D3D11_INPUT_ELEMENT_DESC; 3] = [
    d3d11::D3D11_INPUT_ELEMENT_DESC {
        SemanticName: "POSITION\0".as_ptr() as *const i8,
        SemanticIndex: 0,
        Format: dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: 0,
        InputSlotClass: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
    d3d11::D3D11_INPUT_ELEMENT_DESC {
        SemanticName: "NORMAL\0".as_ptr() as *const i8,
        SemanticIndex: 0,
        Format: dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: d3d11::D3D11_APPEND_ALIGNED_ELEMENT,
        InputSlotClass: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
    d3d11::D3D11_INPUT_ELEMENT_DESC {
        SemanticName: "TEXCOORD\0".as_ptr() as *const i8,
        SemanticIndex: 0,
        Format: dxgiformat::DXGI_FORMAT_R32G32_FLOAT,
        InputSlot: 0,
        AlignedByteOffset: d3d11::D3D11_APPEND_ALIGNED_ELEMENT,
        InputSlotClass: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
        InstanceDataStepRate: 0,
    },
];

pub const SCREEN_QUAD_VERTICES: [Vertex; 3] = [
    Vertex {
        position: Vec3::new(-1.0, 1.0, 0.5),
        normal: Vec3::new(0.0, 0.0, -1.0),
        uv: Vec2::new(0.0, 0.0),
    },
    Vertex {
        position: Vec3::new(1.0, 1.0, 0.5),
        normal: Vec3::new(0.0, 0.0, -1.0),
        uv: Vec2::new(1.0, 0.0),
    },
    Vertex {
        position: Vec3::new(-1.0, -1.0, 0.5),
        normal: Vec3::new(0.0, 0.0, -1.0),
        uv: Vec2::new(0.0, 1.0),
    },
];

pub fn create_input_layout(
    device: &ComPtr<d3d11::ID3D11Device>,
    layouts: &[d3d11::D3D11_INPUT_ELEMENT_DESC],
    vertex_shader_binary: &[u8],
) -> Result<ComPtr<d3d11::ID3D11InputLayout>> {
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

pub fn create_vertex_buffer(
    device: &ComPtr<d3d11::ID3D11Device>,
    vertices: &[Vertex],
) -> Result<ComPtr<d3d11::ID3D11Buffer>> {
    create_buffer(
        device,
        vertices,
        d3d11::D3D11_USAGE_DEFAULT,
        d3d11::D3D11_BIND_VERTEX_BUFFER,
        0,
        "Vertex",
    )
}

/// 型付き Constant Buffer
pub struct ConstantBuffer<T> {
    pub(crate) buffer: ComPtr<d3d11::ID3D11Buffer>,
    modifiable: bool,
    inner_type: PhantomData<fn() -> T>,
}

impl<T> ConstantBuffer<T> {
    pub fn new(device: &ComPtr<d3d11::ID3D11Device>, initial: &T) -> Result<ConstantBuffer<T>> {
        let buffer = create_buffer(
            device,
            &[initial],
            d3d11::D3D11_USAGE_DYNAMIC,
            d3d11::D3D11_BIND_CONSTANT_BUFFER,
            d3d11::D3D11_CPU_ACCESS_WRITE,
            "Constant",
        )?;

        Ok(ConstantBuffer {
            buffer,
            modifiable: true,
            inner_type: Default::default(),
        })
    }

    pub fn update(&mut self, context: &Context, data: &T) {
        // TODO: もしかして &mut じゃなくてもよくない？
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

/// Index Buffer の要素に使える型が実装する trait 。
pub trait IndexInteger {
    /// DXGI_FORMAT 定数を返す。
    fn dxgi_format() -> dxgiformat::DXGI_FORMAT;
}

impl IndexInteger for u8 {
    fn dxgi_format() -> dxgiformat::DXGI_FORMAT {
        dxgiformat::DXGI_FORMAT_R16_UINT
    }
}

impl IndexInteger for u16 {
    fn dxgi_format() -> dxgiformat::DXGI_FORMAT {
        dxgiformat::DXGI_FORMAT_R16_UINT
    }
}

impl IndexInteger for u32 {
    fn dxgi_format() -> dxgiformat::DXGI_FORMAT {
        dxgiformat::DXGI_FORMAT_R32_UINT
    }
}

/// 型付きの Index Buffer
pub struct IndexBuffer<T: IndexInteger> {
    pub(crate) buffer: ComPtr<d3d11::ID3D11Buffer>,
    inner_type: PhantomData<fn() -> T>,
}

impl<T: IndexInteger> IndexBuffer<T> {
    pub fn new(device: &ComPtr<d3d11::ID3D11Device>, indices: &[T]) -> Result<IndexBuffer<T>> {
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
            inner_type: Default::default(),
        })
    }
}

/// `ID3D11Buffer` を作成する。
fn create_buffer<T>(
    device: &ComPtr<d3d11::ID3D11Device>,
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
        pSysMem: data.as_ptr() as *const T as *const c_void,
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
