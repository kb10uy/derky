//! Windows COM の型サポート

use std::{ops::Deref, ptr::NonNull};

use anyhow::{bail, Result};
use winapi::{
    shared::{
        minwindef::ULONG,
        winerror::{HRESULT, SUCCEEDED},
    },
    um::unknwnbase::IUnknown,
    Interface,
};

/// ComPtr のラッパー
#[derive(Debug)]
pub struct ComPtr<T: Interface>(NonNull<T>);

#[allow(dead_code)]
impl<T: Interface> ComPtr<T> {
    pub fn new(pointer: *mut T) -> Option<ComPtr<T>> {
        NonNull::new(pointer).map(ComPtr)
    }

    /// ポインタを取得する。この際 AddRef() は呼ばれない。
    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    /// move することによってポインタを取得する。
    pub fn into_ptr(self) -> *mut T {
        self.0.as_ptr()
    }
}

impl<T: Interface> Deref for ComPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: Interface> Clone for ComPtr<T> {
    fn clone(&self) -> Self {
        unsafe {
            let unknown = &*(self.0.as_ptr() as *mut IUnknown);
            unknown.AddRef();
        }
        ComPtr(self.0)
    }
}

impl<T: Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let unknown = &*(self.0.as_ptr() as *mut IUnknown);
            unknown.Release();
        }
    }
}

/// HRESULT から Result への変換
pub trait HresultErrorExt {
    fn err(self) -> Result<()>;
}

impl HresultErrorExt for HRESULT {
    fn err(self) -> Result<()> {
        if SUCCEEDED(self) {
            Ok(())
        } else {
            bail!("HRESULT error value: 0x{:X}", self);
        }
    }
}

impl HresultErrorExt for ULONG {
    fn err(self) -> Result<()> {
        if self == 0 {
            Ok(())
        } else {
            bail!("ULONG error value: 0x{:X}", self);
        }
    }
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
        let $i = if let Some(comptr) = ComPtr::new($i) {
            comptr
        } else {
            anyhow::bail!("{} is NULL", stringify!($i));
        };
    )* }
}
