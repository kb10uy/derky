//! Safe operation of Windows COM.

use std::{ops::Deref, ptr::NonNull};

pub use anyhow::{bail, Result};
use winapi::{
    shared::{
        minwindef::ULONG,
        winerror::{HRESULT, SUCCEEDED},
    },
    um::unknwnbase::IUnknown,
    Interface,
};

/// Wraps COM Object.
#[derive(Debug)]
pub struct ComPtr<T: Interface>(NonNull<T>);

#[allow(dead_code)]
impl<T: Interface> ComPtr<T> {
    pub fn new(pointer: *mut T) -> Option<ComPtr<T>> {
        NonNull::new(pointer).map(ComPtr)
    }

    /// Returns the raw pointer.
    /// AddRef() won't be called internally.
    pub fn as_ptr(&self) -> *mut T {
        self.0.as_ptr()
    }

    /// Moves and Returns the raw pointer.
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

/// The extension trait for `HRESULT` type.
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

