//! Contains texture image operations.

use std::path::Path;

use anyhow::{bail, Result};
use exr::prelude::rgba_image::*;
use image::{imageops::resize, imageops::FilterType, GenericImageView, Primitive, Rgba};
use log::{debug, info};

/// Represents a RGBA image data with raw element array and dimensions.
pub struct RgbaImageData<T: Copy> {
    data: Box<[T]>,
    width: usize,
    height: usize,
}

impl<T: 'static + Copy> RgbaImageData<T> {
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

    /// Creates a new image from raw data.
    pub fn new(data: &[T], width: usize, height: usize) -> Result<RgbaImageData<T>> {
        let length = data.len();
        match length {
            x if x % 4 != 0 => bail!("The length of data is not multiple of 4"),
            x if x / 4 < width * height => bail!("The data is not enough for the dimensions"),
            _ => {
                let new_data = Vec::from(data);
                Ok(RgbaImageData {
                    data: new_data.into_boxed_slice(),
                    width,
                    height,
                })
            }
        }
    }

    /// Resizes (scale up) to dimensions which are power of 2.
    pub fn resize_to_power_of_2(&self) -> RgbaImageData<T>
    where
        T: Primitive,
    {
        /// Rounds up to power of 2.
        fn round(x: usize) -> usize {
            if x.count_ones() == 1 {
                x
            } else {
                1 << (32 - x.leading_zeros())
            }
        };

        let new_width = round(self.width);
        let new_height = round(self.height);

        let new_image = resize(
            self,
            new_width as u32,
            new_height as u32,
            FilterType::Lanczos3,
        );

        RgbaImageData {
            data: new_image.into_raw().into_boxed_slice(),
            width: new_width,
            height: new_height,
        }
    }
}

impl<T: 'static + Primitive> GenericImageView for RgbaImageData<T> {
    type Pixel = Rgba<T>;
    type InnerImageView = Self;

    fn dimensions(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, self.width as u32, 0, self.height as u32)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        let base_index = (y as usize * self.width + x as usize) * 4;
        Rgba([
            self.data[base_index + 0],
            self.data[base_index + 1],
            self.data[base_index + 2],
            self.data[base_index + 3],
        ])
    }

    fn inner(&self) -> &Self::InnerImageView {
        self
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
