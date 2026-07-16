pub mod bricks;
pub mod color;
pub mod graphics;
pub mod m3;
pub mod render;
pub mod save;
pub mod stitcher;
pub mod util;
pub mod world_load;

pub use render::Renderer;
pub use save::SaveData;
pub use stitcher::TileStitcher;

/// Encodes tightly-packed RGBA pixels as a PNG.
pub fn encode_png(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    let mut png: Vec<u8> = Vec::new();
    image::png::PngEncoder::new(&mut png)
        .encode(pixels, width, height, image::ColorType::Rgba8)
        .map_err(|e| format!("Error encoding to png: {}", e))?;
    Ok(png)
}
