//! Contains texture image operations.

use std::path::Path;

use anyhow::Result;
use exr::prelude::rgba_image::*;
use log::{debug, info};

/// Represents a RGBA image data with raw element array and dimensions.
pub struct RgbaImageData<T> {
    data: Box<[T]>,
    width: usize,
    height: usize,
}

impl<T> RgbaImageData<T> {
    /// Gets the dimension of this image.
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Gets the reference to the raw data of this image.
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Consumes this instance and gets the raw data of this image.
    pub fn into_data(self) -> Box<[T]> {
        self.data
    }
}

/// Loads a LDR (PNG, JPEG, and DXT) image.
pub fn load_ldr_image(filename: impl AsRef<Path>) -> Result<RgbaImageData<u8>> {
    let filename = filename.as_ref();

    debug!("Loading LDR image {:?}", filename);
    let original_image = image::open(filename)?.into_rgba();
    let dimensions = original_image.dimensions();
    let data = original_image.into_raw().into_boxed_slice();

    info!("Loaded successfully; dimensions are {:?}", dimensions);
    Ok(RgbaImageData {
        data,
        width: dimensions.0 as usize,
        height: dimensions.1 as usize,
    })
}

/// Loads a HDR (OpenEXR only) image.
pub fn load_hdr_image(filename: impl AsRef<Path>) -> Result<RgbaImageData<f32>> {
    let filename = filename.as_ref();

    debug!("Loading HDR image {:?}", filename);
    let (_, (data, width, height)) = ImageInfo::read_pixels_from_file(
        filename,
        read_options::high(),
        |info| {
            let w = info.resolution.width();
            let h = info.resolution.height();
            let image = vec![0f32; w * h * 4];
            (image, w, h)
        },
        |(image, w, _), pos, pixel| {
            let base_index = (pos.y() * *w + pos.x()) * 4;
            let pixel_array: [f32; 4] = pixel.into();
            for i in 0..4 {
                image[base_index + i] = pixel_array[i];
            }
        },
    )?;

    info!("Loaded successfully; dimensions are {:?}", (width, height));
    Ok(RgbaImageData {
        data: data.into_boxed_slice(),
        width,
        height,
    })
}
