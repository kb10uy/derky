//! Defines abstract data and operations for generic data,
//! and general purpose rendering framework.

/// Common operations
pub mod common {
    pub mod environment;
    pub mod model;
    pub mod texture;
}

/// Rendering framework with Direct3D 11.
pub mod d3d11 {
    pub mod buffer;
    mod com_support;
    pub mod context;
    pub mod shader;
    pub mod texture;
    pub mod vertex;
}

/// NULL を生成する。
#[macro_export]
macro_rules! null {
    ($t: ty) => {
        0 as *mut $t
    };
}

/// *mut T から NonNull<T> に変換する。
/// いずれかに NULL が含まれていた場合 Err でベイルアウトする。
#[macro_export]
macro_rules! comptrize {
    ($($i:ident),* $(,)?) => { $(
        let $i = if let Some(comptr) = $crate::d3d11::com_support::ComPtr::new($i) {
            comptr
        } else {
            anyhow::bail!("{} is NULL", stringify!($i));
        };
    )* }
}
