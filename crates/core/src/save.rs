use crate::brick::Brick;
use crate::color::*;
use crate::bricks::*;
use crate::graphics::push_shape;
use crate::m3;
use crate::render::Renderer;
use crate::util;
use crate::world_load;

use std::collections::HashSet;

use brickadia::read::SaveReader;
use brickadia::save::{Rotation, Direction, BrickColor};

// Geometry batches flush to the GPU around this many indices. Batch AABBs are
// what viewport culling skips, so this is the culling granularity; smaller
// batches cull tighter but cost more draw calls.
const BATCH_INDEX_TARGET: usize = 262_144;

// Coverage grid for occlusion culling: cells this many save units square
// (half a stud), coarsened as needed to cap the grid dimensions on huge maps.
const CULL_CELL_SIZE: i32 = 5;
const CULL_MAX_GRID_DIM: i32 = 4096;

// Outlines extend past brick footprints; batch AABBs pad by this much
const AABB_PAD: f32 = 1.0;

#[derive(PartialEq, Eq, Hash)]
struct BrickShape {
    name_index: u32,
    size: (u16, u16),
    position: (i32, i32),
    rotation: Rotation,
    direction: Direction,
}

impl BrickShape {
    fn of(brick: &Brick) -> BrickShape {
        BrickShape {
            name_index: brick.asset_name_index,
            position: (brick.position.0, brick.position.1),
            size: (brick.size.0, brick.size.1),
            rotation: brick.rotation.clone(),
            direction: brick.direction.clone(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GeometryMode {
    Map { outlines: bool, fills: bool },
    Heightmap,
}

// One spatial chunk of the save. brdb worlds provide these natively; other
// formats load as a single chunk covering everything.
struct MapChunk {
    // Vertical layer, ascending draw order. Bricks crossing a layer boundary
    // can draw out of height order against the neighboring layer — the price
    // of chunked drawing.
    layer: i16,
    // Sorted by top surface, so draw order within a chunk is exact
    bricks: Vec<Brick>,
}

/// A save's render-ready bricks, grouped into spatial chunks. Populated
/// incrementally through [`SaveLoading`]; complete data is also available in
/// one call via [`SaveData::load`].
pub struct SaveData {
    // Processing order: top layer first, the order the occlusion grid needs.
    // Draw order (bottom layer first) comes from batch sort keys.
    chunks: Vec<MapChunk>,
    pub brick_assets: Vec<String>,
    pub description: String,
    pub brick_count: i32,
    /// View center and rotation pivot. Fixed before bricks stream in, so the
    /// view stays put while a save loads progressively.
    pub centroid: (i32, i32),
    /// Extent of brick footprints relative to the centroid; grows as chunks
    /// stream in.
    pub bounds: (i32, i32, i32, i32),
    // Absolute superset of all brick footprints, fixed up front, sizing the
    // occlusion grid
    grid_bounds: (i32, i32, i32, i32),
    /// Duplicate bricks dropped during load (same footprint stacked exactly)
    pub discarded: usize,
    // Cross-chunk duplicate suppression; only lives while loading
    dedupe: HashSet<BrickShape>,
}

impl SaveData {
    /// Parses any supported save format (.brs, .brz, .brdb) from bytes in one
    /// call.
    pub fn load(body: &[u8]) -> Result<SaveData, String> {
        let mut loading = SaveLoading::open(body)?;
        while loading.step()? {}
        loading.finish()
    }

    fn new(
        description: String,
        brick_count: i32,
        centroid: (i32, i32),
        grid_bounds: (i32, i32, i32, i32),
    ) -> SaveData {
        SaveData {
            chunks: Vec::new(),
            brick_assets: Vec::new(),
            description,
            brick_count,
            centroid,
            bounds: (i32::MAX, i32::MAX, i32::MIN, i32::MIN),
            grid_bounds,
            discarded: 0,
            dedupe: HashSet::new(),
        }
    }

    // Sorts, deduplicates, and stores one chunk's bricks
    fn push_chunk(&mut self, layer: i16, mut bricks: Vec<Brick>) {
        bricks.sort_unstable_by_key(util::top_surface);

        // Walked top-down so the topmost copy — drawn last — survives
        let mut keep = vec![true; bricks.len()];
        for i in (0..bricks.len()).rev() {
            keep[i] = self.dedupe.insert(BrickShape::of(&bricks[i]));
        }
        let before = bricks.len();
        let mut it = keep.iter();
        bricks.retain(|_| *it.next().unwrap());
        self.discarded += before - bricks.len();

        for brick in &bricks {
            let size = util::sizer(brick);
            self.bounds.0 = self.bounds.0.min(brick.position.0 - size.0 as i32 - self.centroid.0);
            self.bounds.1 = self.bounds.1.min(brick.position.1 - size.1 as i32 - self.centroid.1);
            self.bounds.2 = self.bounds.2.max(brick.position.0 + size.0 as i32 - self.centroid.0);
            self.bounds.3 = self.bounds.3.max(brick.position.1 + size.1 as i32 - self.centroid.1);
        }

        if !bricks.is_empty() {
            self.chunks.push(MapChunk { layer, bricks });
        }
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Screen-space transform for the given viewport, pan (world units),
    /// scale, and rotation, centered on the save's centroid.
    pub fn view_matrix(&self, size_x: f32, size_y: f32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> [f32; 9] {
        let mut matrix = m3::projection(size_x, size_y);
        matrix = m3::translate(matrix, size_x / 2.0, size_y / 2.0);
        matrix = m3::scale(matrix, scale, scale);
        matrix = m3::rotate(matrix, rotation);
        matrix = m3::translate(matrix, pan_x - self.centroid.0 as f32, pan_y - self.centroid.1 as f32);
        matrix
    }

    /// Rebuilds all geometry for the currently loaded chunks. Returns the
    /// number of bricks skipped by occlusion culling.
    pub fn build_geometry(&self, mode: GeometryMode, renderer: &mut Renderer) -> Result<usize, String> {
        renderer.clear_batches();
        let mut state = GeometryState::new(self, mode);
        state.build_pending(self, renderer)?;
        state.flush(renderer);
        Ok(state.culled)
    }

    fn height_extent(&self) -> (i32, i32) {
        let mut min_height = i32::MAX;
        let mut max_height = i32::MIN;
        for chunk in &self.chunks {
            for brick in &chunk.bricks {
                let size = util::sizer(brick);
                let top = brick.position.2 + size.2 as i32;
                let bot = brick.position.2 - size.2 as i32;
                min_height = std::cmp::min(min_height, bot);
                max_height = std::cmp::max(max_height, top);
            }
        }
        (min_height, max_height)
    }
}

enum LoadSource {
    // Everything parsed up front; one pending chunk
    Whole(Option<(i16, Vec<Brick>)>),
    Stream(Box<dyn world_load::ChunkSource>),
}

/// Incremental save loading: `open`, then `step` until it returns false,
/// then `finish`. The partial [`SaveData`] is readable throughout, so
/// geometry can build and draw while later chunks are still parsing.
pub struct SaveLoading {
    save: SaveData,
    source: LoadSource,
}

impl SaveLoading {
    pub fn open(body: &[u8]) -> Result<SaveLoading, String> {
        if body.starts_with(b"BRZ") {
            Ok(Self::streamed(world_load::open_brz(body)?))
        } else if body.starts_with(b"SQLite format 3\0") {
            Ok(Self::streamed(world_load::open_brdb(body)?))
        } else {
            Self::open_brs(body)
        }
    }

    fn streamed(stream: Box<dyn world_load::ChunkSource>) -> SaveLoading {
        let gb = stream.grid_bounds();
        // Center the view on the chunk extent; the true bounds aren't known
        // until every chunk has streamed in
        let centroid = ((gb.0 + gb.2) / 2, (gb.1 + gb.3) / 2);
        let save = SaveData::new(stream.description().to_string(), stream.brick_count(), centroid, gb);
        SaveLoading { save, source: LoadSource::Stream(stream) }
    }

    fn open_brs(body: &[u8]) -> Result<SaveLoading, String> {
        let mut reader = SaveReader::new(body)
            .map_err(|_| "brickadia-rs error creating save reader".to_string())?;
        let save = reader.read_all()
            .map_err(|_| "brickadia-rs error reading file".to_string())?;

        // Get color list as display-ready sRGB (brs stores linear values)
        let mut colors: Vec<Color> = save.header2.colors.iter().map(convert_color).collect();
        for color in &mut colors {
            color.convert_to_srgb();
        }

        let brick_assets = save.header2.brick_assets;
        let bricks: Vec<Brick> = save.bricks.iter()
            .filter_map(|brick| {
                let color = match &brick.color {
                    BrickColor::Index(color_index) => colors[*color_index as usize],
                    BrickColor::Unique(color) => {
                        let mut color = convert_color(color);
                        color.convert_to_srgb();
                        color
                    }
                };
                util::slim_brick(brick, &brick_assets, color.to_bytes())
            })
            .collect();

        if bricks.is_empty() {
            return Err("save contains no visible bricks".to_string());
        }

        let centroid = util::calculate_centroid(&bricks);
        let grid_bounds = util::footprint_bounds(&bricks);

        let mut data = SaveData::new(
            save.header1.description,
            save.header1.brick_count as i32,
            centroid,
            grid_bounds,
        );
        data.brick_assets = brick_assets;

        Ok(SaveLoading { save: data, source: LoadSource::Whole(Some((0, bricks))) })
    }

    /// The partially loaded save (bounds and chunks grow as steps complete).
    pub fn save(&self) -> &SaveData {
        &self.save
    }

    /// Fraction loaded, 0.0..=1.0.
    pub fn progress(&self) -> f32 {
        match &self.source {
            LoadSource::Whole(pending) => if pending.is_some() { 0.0 } else { 1.0 },
            LoadSource::Stream(stream) => stream.progress(),
        }
    }

    /// Parses the next chunk. Returns false once everything is loaded.
    pub fn step(&mut self) -> Result<bool, String> {
        let next = match &mut self.source {
            LoadSource::Whole(pending) => pending.take(),
            LoadSource::Stream(stream) => stream.next_chunk(&mut self.save.brick_assets)?,
        };
        match next {
            Some((layer, bricks)) => {
                self.save.push_chunk(layer, bricks);
                Ok(!matches!(self.source, LoadSource::Whole(_)))
            }
            None => Ok(false),
        }
    }

    pub fn finish(self) -> Result<SaveData, String> {
        let mut save = self.save;
        if save.is_empty() {
            return Err("save contains no visible bricks".to_string());
        }
        save.dedupe = HashSet::new();
        Ok(save)
    }
}

// Boolean coverage over the save's footprint for occlusion culling
struct CoverGrid {
    min_x: i32,
    min_y: i32,
    cell: i32,
    cols: usize,
    rows: usize,
    covered: Vec<bool>,
}

impl CoverGrid {
    fn new(bounds: (i32, i32, i32, i32)) -> CoverGrid {
        let (min_x, min_y, max_x, max_y) = bounds;
        let extent = std::cmp::max(max_x - min_x, max_y - min_y).max(1);
        let cell = std::cmp::max(CULL_CELL_SIZE, (extent + CULL_MAX_GRID_DIM - 1) / CULL_MAX_GRID_DIM);
        let cols = ((max_x - min_x) / cell + 1) as usize;
        let rows = ((max_y - min_y) / cell + 1) as usize;
        CoverGrid { min_x, min_y, cell, cols, rows, covered: vec![false; cols * rows] }
    }

    // Footprint in grid cell coordinates; None when degenerate or outside
    fn cells(&self, brick: &Brick) -> Option<(i32, i32, i32, i32)> {
        let size = util::sizer(brick);
        let x1 = brick.position.0 - size.0 as i32 - self.min_x;
        let y1 = brick.position.1 - size.1 as i32 - self.min_y;
        let x2 = x1 + 2 * size.0 as i32;
        let y2 = y1 + 2 * size.1 as i32;
        if x2 <= x1 || y2 <= y1 || x1 < 0 || y1 < 0 {
            return None;
        }
        if (x2 - 1) / self.cell >= self.cols as i32 || (y2 - 1) / self.cell >= self.rows as i32 {
            return None;
        }
        Some((x1, y1, x2, y2))
    }

    fn fully_covered(&self, brick: &Brick) -> bool {
        let Some((x1, y1, x2, y2)) = self.cells(brick) else {
            return false;
        };
        for r in (y1 / self.cell)..=((y2 - 1) / self.cell) {
            for c in (x1 / self.cell)..=((x2 - 1) / self.cell) {
                if !self.covered[r as usize * self.cols + c as usize] {
                    return false;
                }
            }
        }
        true
    }

    // Mark only cells lying entirely inside the footprint
    fn cover(&mut self, brick: &Brick) {
        let Some((x1, y1, x2, y2)) = self.cells(brick) else {
            return;
        };
        for r in ((y1 + self.cell - 1) / self.cell)..(y2 / self.cell) {
            for c in ((x1 + self.cell - 1) / self.cell)..(x2 / self.cell) {
                self.covered[r as usize * self.cols + c as usize] = true;
            }
        }
    }
}

/// Incremental geometry builder. Consumes chunks in the save's processing
/// order (top layer first), maintaining the occlusion grid across chunks, and
/// uploads batches keyed for bottom-first draw order. Survives across
/// [`SaveLoading::step`] calls so geometry can build as chunks stream in.
pub struct GeometryState {
    mode: GeometryMode,
    grid: CoverGrid,
    // Fixed at creation; only meaningful for heightmap mode, which is why a
    // mid-stream heightmap needs a final rebuild once loading completes
    height_extent: (i32, i32),
    next_chunk: usize,
    pub culled: usize,
    staging_vertices: Vec<u8>,
    staging_indices: Vec<u32>,
    staging_aabb: (f32, f32, f32, f32),
    staging_layer: i16,
}

impl GeometryState {
    pub fn mode(&self) -> GeometryMode {
        self.mode
    }

    pub fn new(save: &SaveData, mode: GeometryMode) -> GeometryState {
        GeometryState {
            mode,
            grid: CoverGrid::new(save.grid_bounds),
            height_extent: match mode {
                GeometryMode::Heightmap => save.height_extent(),
                _ => (0, 1),
            },
            next_chunk: 0,
            culled: 0,
            staging_vertices: Vec::new(),
            staging_indices: Vec::new(),
            staging_aabb: (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
            staging_layer: 0,
        }
    }

    /// Builds geometry for any chunks added to the save since the last call.
    pub fn build_pending(&mut self, save: &SaveData, renderer: &mut Renderer) -> Result<(), String> {
        while self.next_chunk < save.chunks.len() {
            self.build_chunk(save, renderer);
            self.next_chunk += 1;
        }
        Ok(())
    }

    fn build_chunk(&mut self, save: &SaveData, renderer: &mut Renderer) {
        let chunk = &save.chunks[self.next_chunk];

        // Batches never span layers — the layer is the draw-order key
        if chunk.layer != self.staging_layer {
            self.flush(renderer);
            self.staging_layer = chunk.layer;
        }

        // Outline-only mode draws no fills, so nothing occludes anything
        let cull = match self.mode {
            GeometryMode::Map { fills, .. } => fills,
            GeometryMode::Heightmap => true,
        };

        // Top-down: a brick is hidden if every coverage cell its footprint
        // touches was fully covered by the rectangular fills of bricks drawn
        // over it. Conservative on both sides — shaped bricks never cover,
        // partial cells never count as covered.
        let mut hidden = vec![false; chunk.bricks.len()];
        if cull {
            for (k, brick) in chunk.bricks.iter().enumerate().rev() {
                if self.grid.fully_covered(brick) {
                    hidden[k] = true;
                    self.culled += 1;
                    continue;
                }
                let name = &save.brick_assets[brick.asset_name_index as usize];
                if is_full_rect(name) {
                    self.grid.cover(brick);
                }
            }
        }

        for (brick, &hide) in chunk.bricks.iter().zip(&hidden) {
            if hide {
                continue;
            }
            let name = &save.brick_assets[brick.asset_name_index as usize];

            match self.mode {
                GeometryMode::Map { outlines, fills } => {
                    if fills {
                        let verts = calculate_brick_vertices(name, brick);
                        push_shape(&mut self.staging_vertices, &mut self.staging_indices, &verts, brick.color);
                    }
                    if outlines {
                        let ol_verts = calculate_brick_outline_vertices(name, brick);
                        push_shape(&mut self.staging_vertices, &mut self.staging_indices, &ol_verts, Color::black().to_bytes());
                    }
                }
                GeometryMode::Heightmap => {
                    let (min_height, max_height) = self.height_extent;
                    let verts = calculate_brick_vertices(name, brick);
                    let relative_height = (brick.position.2 - min_height) as f32 / (max_height - min_height).max(1) as f32;
                    let level = (relative_height * 255.0) as u8;
                    push_shape(&mut self.staging_vertices, &mut self.staging_indices, &verts, [level, level, level, 255]);
                }
            }

            let size = util::sizer(brick);
            self.staging_aabb.0 = self.staging_aabb.0.min((brick.position.0 - size.0 as i32) as f32 - AABB_PAD);
            self.staging_aabb.1 = self.staging_aabb.1.min((brick.position.1 - size.1 as i32) as f32 - AABB_PAD);
            self.staging_aabb.2 = self.staging_aabb.2.max((brick.position.0 + size.0 as i32) as f32 + AABB_PAD);
            self.staging_aabb.3 = self.staging_aabb.3.max((brick.position.1 + size.1 as i32) as f32 + AABB_PAD);

            if self.staging_indices.len() >= BATCH_INDEX_TARGET {
                self.flush(renderer);
            }
        }
    }

    /// Uploads any staged geometry as a batch. Call after `build_pending` so
    /// partially filled batches reach the screen too.
    pub fn flush(&mut self, renderer: &mut Renderer) {
        if self.staging_indices.is_empty() {
            return;
        }
        renderer.upload_batch(
            self.staging_layer as i32,
            self.staging_aabb,
            &self.staging_vertices,
            &self.staging_indices,
        );
        self.staging_vertices.clear();
        self.staging_indices.clear();
        self.staging_aabb = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    }
}
