use anyhow::{anyhow, Result};
use std::{io::Cursor, path::Path};

mod processing;

/// Width of the map. Cell every 6 minutes, 180 degrees of latitude.
/// Rounded to nearest multiple of 256 for alignment reasons.
const MAP_WIDTH: usize = 3584;

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
        // ensure texture contents are consistent with internal state
        self.graphics
            .set_source_texture_contents(&self.buffer)
            .await?;

        for _ in 0..count {
            self.graphics.apply_shader()?;
        }

        self.graphics.get_texture_contents(&mut self.buffer).await
    }

    /// Saves the current map state as an 8-bit RGBA image.
    pub fn get_state_clone(&self) -> Vec<u8> {
        self.buffer.clone()
    }

    pub fn save_raw_to_image<P: AsRef<Path>>(raw_state: Vec<u8>, path: P) -> Result<()> {
        let image_data = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
            MAP_WIDTH.try_into()?,
            MAP_HEIGHT.try_into()?,
            raw_state,
        )
        .ok_or_else(|| anyhow!("couldn't convert state to image"))?;
        image_data.save(path.as_ref()).map_err(Into::into)
    }

    /// Renders the current state to the provided rectangle/view.
    ///
    /// Currently just samples the state but eventually will average over regions.
    pub fn render_cropped_state(&self, section: super::message::Rect) -> Result<Vec<u8>> {
        let state = self.get_state_clone();
        let image_data = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
            MAP_WIDTH.try_into()?,
            MAP_HEIGHT.try_into()?,
            state,
        )
        .ok_or_else(|| anyhow!("couldn't convert state to image"))?;
        let image = image::DynamicImage::ImageRgba8(image_data);

        let super::message::Rect {
            top_left,
            bottom_right,
        } = section;

        // NOTE: top_left/bottom_right will have different components because of how zooming works

        let (x, y) = latlong_to_pixel_coords(top_left);
        let (br_x, br_y) = latlong_to_pixel_coords(bottom_right);
        log::debug!("{x}, {y} -> {br_x}, {br_y}");
        let cropped = image.crop_imm(x, y, br_x - x, br_y - y);

        let scaled = cropped.resize_exact(22, 40, image::imageops::FilterType::CatmullRom);

        let mut output_cursor = Cursor::new(Vec::new());
        scaled.write_to(&mut output_cursor, image::ImageFormat::Png)?;

        Ok(output_cursor.into_inner())
    }
}

fn latlong_to_pixel_coords(latlong: crate::message::LatLong) -> (u32, u32) {
    let x = ((latlong.long + 180.) / 360.) * MAP_WIDTH as f64;
    let y = ((latlong.lat + 90.) / 180.) * MAP_HEIGHT as f64;

    (
        (x as u32).clamp(0, MAP_WIDTH as u32 - 1),
        (y as u32).clamp(0, MAP_HEIGHT as u32 - 1),
    )
}
