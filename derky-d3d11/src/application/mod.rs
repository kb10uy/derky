mod environment;
mod model;

pub use model::{load_obj, ModelVertex, MODEL_VERTEX_LAYOUT};

use crate::rendering::{DepthStencil, RenderTarget, Device};

use anyhow::Result;

pub struct Application {
    g_buffer: Box<[RenderTarget]>,
}

impl Application {
    pub fn new(device: &Device) -> Result<Application> {
        todo!()
    }
}
