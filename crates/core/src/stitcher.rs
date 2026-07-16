/// Stitches raw RGBA screenshot tiles into one PNG. Tiles are blitted
/// straight into the full-size pixel buffer as they arrive, so peak memory is
/// the final image plus one tile.
pub struct TileStitcher {
    pixels: Vec<u8>,
    tile_width: u32,
    tile_height: u32,
    rows: u32,
    cols: u32,
}

impl Default for TileStitcher {
    fn default() -> TileStitcher {
        TileStitcher {
            pixels: Vec::new(),
            tile_width: 0,
            tile_height: 0,
            rows: 0,
            cols: 0,
        }
    }
}

impl TileStitcher {
    pub fn set_layout(&mut self, tile_width: u32, tile_height: u32, rows: u32, cols: u32) {
        self.tile_width = tile_width;
        self.tile_height = tile_height;
        self.rows = rows;
        self.cols = cols;
        self.pixels.clear();
        self.pixels.resize((tile_width * cols * tile_height * rows * 4) as usize, 0);
    }

    /// Copies one tile of tightly-packed RGBA pixels into place.
    pub fn push_pixels(&mut self, tile: &[u8], row: u32, col: u32) -> Result<(), String> {
        if row >= self.rows || col >= self.cols {
            return Err("tile out of bounds".to_string());
        }
        if tile.len() != (self.tile_width * self.tile_height * 4) as usize {
            return Err("unexpected tile size".to_string());
        }

        let image_width = (self.cols * self.tile_width) as usize;
        let tile_row_bytes = (self.tile_width * 4) as usize;
        for y in 0..self.tile_height as usize {
            let src = y * tile_row_bytes;
            let dst = ((row * self.tile_height) as usize + y) * image_width * 4
                + (col * self.tile_width) as usize * 4;
            self.pixels[dst..dst + tile_row_bytes].copy_from_slice(&tile[src..src + tile_row_bytes]);
        }

        Ok(())
    }

    /// Encodes the stitched image as a PNG and frees the pixel buffer.
    pub fn encode_png(&mut self) -> Result<Vec<u8>, String> {
        let png = crate::encode_png(
            &self.pixels,
            self.cols * self.tile_width,
            self.rows * self.tile_height,
        )?;
        self.pixels = Vec::new();
        Ok(png)
    }

    #[cfg(test)]
    fn pixel_at(&self, x: u32, y: u32) -> [u8; 4] {
        let image_width = self.cols * self.tile_width;
        let i = ((y * image_width + x) * 4) as usize;
        self.pixels[i..i + 4].try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A tile filled with one color, used to check it lands in the right place
    fn solid_tile(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
        color.iter().copied().cycle().take((width * height * 4) as usize).collect()
    }

    #[test]
    fn places_tiles_in_a_grid() {
        let (tw, th) = (2, 3);
        let mut stitcher = TileStitcher::default();
        stitcher.set_layout(tw, th, 2, 2);

        let colors = [[1, 0, 0, 255], [2, 0, 0, 255], [3, 0, 0, 255], [4, 0, 0, 255]];
        for row in 0..2 {
            for col in 0..2 {
                let color = colors[(row * 2 + col) as usize];
                stitcher.push_pixels(&solid_tile(tw, th, color), row, col).unwrap();
            }
        }

        // Each quadrant should hold its own tile's color, proving row/col map
        // to the right offsets rather than being transposed or overlapping
        for row in 0..2 {
            for col in 0..2 {
                let expected = colors[(row * 2 + col) as usize];
                for y in 0..th {
                    for x in 0..tw {
                        assert_eq!(
                            stitcher.pixel_at(col * tw + x, row * th + y),
                            expected,
                            "tile at row {} col {}, pixel {},{}", row, col, x, y
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn preserves_pixel_order_within_a_tile() {
        let mut stitcher = TileStitcher::default();
        stitcher.set_layout(2, 2, 1, 1);

        // Distinct value per pixel so any row/column flip is visible
        let tile: Vec<u8> = (0..4u8).flat_map(|i| [i, 0, 0, 255]).collect();
        stitcher.push_pixels(&tile, 0, 0).unwrap();

        assert_eq!(stitcher.pixel_at(0, 0), [0, 0, 0, 255]);
        assert_eq!(stitcher.pixel_at(1, 0), [1, 0, 0, 255]);
        assert_eq!(stitcher.pixel_at(0, 1), [2, 0, 0, 255]);
        assert_eq!(stitcher.pixel_at(1, 1), [3, 0, 0, 255]);
    }

    #[test]
    fn rejects_out_of_bounds_and_missized_tiles() {
        let mut stitcher = TileStitcher::default();
        stitcher.set_layout(2, 2, 1, 1);

        assert!(stitcher.push_pixels(&solid_tile(2, 2, [0; 4]), 1, 0).is_err());
        assert!(stitcher.push_pixels(&solid_tile(2, 2, [0; 4]), 0, 1).is_err());
        assert!(stitcher.push_pixels(&solid_tile(2, 1, [0; 4]), 0, 0).is_err());
    }
}
