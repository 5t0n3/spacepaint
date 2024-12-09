use anyhow::{anyhow, Result};
use std::path::Path;

mod processing;

/// Width of the map. Cell every 6 minutes, 180 degrees of latitude.
// const MAP_WIDTH: usize = 360 * 10;
const MAP_WIDTH: usize = 3584; // TEMPORARY, ALIGNMENT

/// Height of the map. Cell every 6 minutes, 180 degres of latitude.
const MAP_HEIGHT: usize = 180 * 10;

/// Bytes per pixel. 1 byte per channel * 4 channels.
const BYTES_PER_PIXEL: usize = 4;

/// Size of the raw state data, in bytes.
const STATE_BYTES: usize = MAP_WIDTH * MAP_HEIGHT * BYTES_PER_PIXEL;

pub struct State {
    /// `wgpu` backend stuff.
    graphics: processing::GraphicsStuff,

    /// Buffer with raw RGBA8 data.
    buffer: Vec<u8>,
}

impl State {
    pub async fn init() -> Result<State> {
        let graphics = processing::GraphicsStuff::init().await?;

        let buffer = Vec::with_capacity(STATE_BYTES);

        // TODO: perlin noise?

        Ok(State { graphics, buffer })
    }

    pub async fn load_from_image<P: AsRef<Path>>(path: P) -> Result<State> {
        let graphics = processing::GraphicsStuff::init().await?;

        let image_data = image::ImageReader::open(path.as_ref())?.decode()?;

        match image_data {
            image::DynamicImage::ImageRgba8(data) => {
                let buffer = data.into_raw();

                // write buffer to underlying texture
                graphics.set_source_texture_contents(&buffer).await?;

                Ok(State { graphics, buffer })
            }
            _ => anyhow::bail!("State images must be 8-bit RGBA"),
        }
    }

    /// Ticks the map state, and updates the internal copy of that state.
    pub async fn tick_state_by_count(&mut self, count: u32) -> Result<()> {
        for i in 0..count {
            println!("tick {i}");
            self.graphics.apply_shader()?;
        }

        self.graphics.get_texture_contents(&mut self.buffer).await
    }

    pub async fn save_state_to_image<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let image_data = image::ImageBuffer::<image::Rgba<u8>, &[u8]>::from_raw(
            MAP_WIDTH.try_into()?,
            MAP_HEIGHT.try_into()?,
            &self.buffer,
        )
        .ok_or_else(|| anyhow!("couldn't convert state to image"))?;
        image_data.save(path.as_ref()).map_err(Into::into)
    }
}
