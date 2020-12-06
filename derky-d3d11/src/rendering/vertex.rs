//! D3D11 の頂点関係

use crate::{
    comptrize, null,
    rendering::{ComPtr, Device, HresultErrorExt},
};

use std::ffi::c_void;

use anyhow::{Context, Result};
use ultraviolet::{Vec2, Vec3, Vec4};
use winapi::{shared::dxgiformat, um::d3d11};

/// D3D11 頂点
pub trait D3d11Vertex {}

/// DXGI_FORMAT 値をもつ型が実装する。
pub trait AsDxgiFormat {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT;
}

impl AsDxgiFormat for f32 {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT = dxgiformat::DXGI_FORMAT_R32_FLOAT;
}

impl AsDxgiFormat for Vec2 {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT = dxgiformat::DXGI_FORMAT_R32G32_FLOAT;
}

impl AsDxgiFormat for Vec3 {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT = dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT;
}

impl AsDxgiFormat for Vec4 {
    const DXGI_FORMAT: dxgiformat::DXGI_FORMAT = dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT;
}

/// D3D11 の頂点とレイアウトを生成する。
#[macro_export]
macro_rules! d3d11_vertex {
    ( $n:ident : $iln:ident { $( $fn:ident : $ft:ty => ( $fs:expr , $fsi:expr ) ),* $(,)? } ) => {
        $crate::__d3d11_vertex_struct!($n { $($fn: $ft),* });
        $crate::__d3d11_vertex_layout!($iln { $($ft => ($fs, $fsi)),* });
    };
}

/// D3D11 の頂点の構造体を定義する。
#[doc(hidden)]
#[macro_export]
macro_rules! __d3d11_vertex_struct {
    ( $n:ident { $( $fn:ident : $ft:ty ),* } ) => {
        #[derive(Debug, Clone)]
        pub struct $n {
            $(pub $fn: $ft,)*
        }

        impl $crate::rendering::D3d11Vertex for $n {}
    };
}

/// D3D11 の頂点の Input Layout を定義する。
#[doc(hidden)]
#[macro_export]
macro_rules! __d3d11_vertex_layout {
    ( $iln:ident { $( $ft:ty => ( $fs:expr , $fsi:expr ) ),* } ) => {
        #[allow(dead_code)]
        pub const $iln: [winapi::um::d3d11::D3D11_INPUT_ELEMENT_DESC; 0 $( + { stringify!($fsi); 1 } )*] =
            $crate::__d3d11_vertex_layout!{$($ft => ($fs, $fsi)),*}
        ;
    };

    {} => {
        []
    };
    { $ft1:ty => ( $fs1:expr , $fsi1:expr ), $( $ft:ty => ( $fs:expr , $fsi:expr ) ),* } => {
        [
            winapi::um::d3d11::D3D11_INPUT_ELEMENT_DESC {
                SemanticName: concat!($fs1, "\0").as_ptr() as *const i8,
                SemanticIndex: $fsi1,
                Format: <$ft1 as $crate::rendering::AsDxgiFormat>::DXGI_FORMAT,
                InputSlot: 0,
                AlignedByteOffset: d3d11::D3D11_APPEND_ALIGNED_ELEMENT,
                InputSlotClass: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            $(
                winapi::um::d3d11::D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: concat!($fs, "\0").as_ptr() as *const i8,
                    SemanticIndex: $fsi,
                    Format: <$ft as $crate::rendering::AsDxgiFormat>::DXGI_FORMAT,
                    InputSlot: 0,
                    AlignedByteOffset: d3d11::D3D11_APPEND_ALIGNED_ELEMENT,
                    InputSlotClass: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                }
            ),*
        ]
    };
}

d3d11_vertex!(Vertex : VERTEX_INPUT_LAYOUT {
    position: Vec3 => ("POSITION", 0),
    normal: Vec3 => ("NORMAL", 0),
    uv: Vec2 => ("TEXCOORD", 0),
});

/// 画面全体のポリゴンの `Vertex` 配列
pub const SCREEN_QUAD_VERTICES: [Vertex; 4] = [
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
    Vertex {
        position: Vec3::new(1.0, -1.0, 0.5),
        normal: Vec3::new(0.0, 0.0, -1.0),
        uv: Vec2::new(1.0, 1.0),
    },
];

/// 画面全体のポリゴンのインデックス配列
pub const SCREEN_QUAD_INDICES: [u32; 6] = [0, 1, 2, 2, 1, 3];

/// Input Layout を作成する。
pub fn create_input_layout(
    device: &Device,
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
