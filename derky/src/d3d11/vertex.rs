//! Contains operations and types for vertices.

use ultraviolet::{Vec2, Vec3, Vec4};
pub use winapi::{
    shared::dxgiformat,
    um::{d3d11, d3dcommon},
};

/// Indicates that this type is reinterpretable as Direct3D 11 vertex.
pub trait D3d11Vertex {}

/// Indicates that this type is reinterpretable as `DXGI_FORMAT` types.
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

/// Generates a vertex type for Direct3D 11 and its Input Layout.
#[macro_export]
macro_rules! d3d11_vertex {
    ( $n:ident : $iln:ident { $( $fn:ident : $ft:ty => ( $fs:expr , $fsi:expr ) ),* $(,)? } ) => {
        $crate::__d3d11_vertex_struct!($n { $($fn: $ft),* });
        $crate::__d3d11_vertex_layout!($iln { $($ft => ($fs, $fsi)),* });
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __d3d11_vertex_struct {
    ( $n:ident { $( $fn:ident : $ft:ty ),* } ) => {
        #[derive(Debug, Clone)]
        pub struct $n {
            $(pub $fn: $ft,)*
        }

        impl $crate::d3d11::vertex::D3d11Vertex for $n {}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __d3d11_vertex_layout {
    ( $iln:ident { $( $ft:ty => ( $fs:expr , $fsi:expr ) ),* } ) => {
        #[allow(dead_code)]
        pub const $iln: [$crate::d3d11::vertex::d3d11::D3D11_INPUT_ELEMENT_DESC; 0 $( + { stringify!($fsi); 1 } )*] =
            $crate::__d3d11_vertex_layout!{$($ft => ($fs, $fsi)),*}
        ;
    };

    {} => {
        []
    };
    { $ft1:ty => ( $fs1:expr , $fsi1:expr ), $( $ft:ty => ( $fs:expr , $fsi:expr ) ),* } => {
        [
            $crate::d3d11::vertex::d3d11::D3D11_INPUT_ELEMENT_DESC {
                SemanticName: concat!($fs1, "\0").as_ptr() as *const i8,
                SemanticIndex: $fsi1,
                Format: <$ft1 as $crate::d3d11::vertex::AsDxgiFormat>::DXGI_FORMAT,
                InputSlot: 0,
                AlignedByteOffset: $crate::d3d11::vertex::d3d11::D3D11_APPEND_ALIGNED_ELEMENT,
                InputSlotClass: $crate::d3d11::vertex::d3d11::D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            $(
                $crate::d3d11::vertex::d3d11::D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: concat!($fs, "\0").as_ptr() as *const i8,
                    SemanticIndex: $fsi,
                    Format: <$ft as $crate::d3d11::vertex::AsDxgiFormat>::DXGI_FORMAT,
                    InputSlot: 0,
                    AlignedByteOffset: $crate::d3d11::vertex::d3d11::D3D11_APPEND_ALIGNED_ELEMENT,
                    InputSlotClass: $crate::d3d11::vertex::d3d11::D3D11_INPUT_PER_VERTEX_DATA,
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

/// `Vertex` array for screen
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

/// Index array for screen
pub const SCREEN_QUAD_INDICES: [u32; 6] = [0, 1, 2, 2, 1, 3];

/// Represents a topology of Vertex Shader input.
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
