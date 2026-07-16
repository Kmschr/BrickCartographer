use std::path::PathBuf;

use brick_cartographer_core::save::GeometryMode;
use brick_cartographer_core::{Renderer, SaveData, TileStitcher};
use clap::Parser;

/// Render a PNG map of a Brickadia save (.brs, .brz, or .brdb).
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Save file to render
    save: PathBuf,

    /// Output PNG path (defaults to the save's name with a .png extension)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pixels per save unit. The website's default view is 0.1
    #[arg(short, long, default_value_t = 0.1)]
    scale: f32,

    /// Draw brick outlines
    #[arg(long)]
    outlines: bool,

    /// Draw bricks without their fill (outlines only)
    #[arg(long)]
    no_fill: bool,

    /// Color bricks by height instead of their own color
    #[arg(long, conflicts_with_all = ["outlines", "no_fill"])]
    heightmap: bool,

    /// Rotation in degrees
    #[arg(short, long, default_value_t = 0.0)]
    rotation: f32,

    /// Margin around the build, in pixels
    #[arg(short, long, default_value_t = 32)]
    margin: u32,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    if args.scale <= 0.0 {
        return Err("scale must be greater than zero".to_string());
    }
    if args.no_fill && !args.outlines {
        return Err("--no-fill needs --outlines, or the map would be empty".to_string());
    }

    let output = args.output.clone().unwrap_or_else(|| args.save.with_extension("png"));

    let body = std::fs::read(&args.save)
        .map_err(|e| format!("Error reading {}: {}", args.save.display(), e))?;
    let save = SaveData::load(&body)?;
    eprintln!("Loaded {} bricks ({} discarded)", save.brick_count, save.discarded);

    let png = pollster::block_on(render(&save, &args))?;
    std::fs::write(&output, png).map_err(|e| format!("Error writing {}: {}", output.display(), e))?;
    eprintln!("Wrote {}", output.display());

    Ok(())
}

async fn render(save: &SaveData, args: &Args) -> Result<Vec<u8>, String> {
    let mut renderer = Renderer::new(None).await?;

    let mode = if args.heightmap {
        GeometryMode::Heightmap
    } else {
        GeometryMode::Map { outlines: args.outlines, fills: !args.no_fill }
    };
    let culled = save.build_geometry(mode, |vertices, indices| {
        renderer.upload_chunk(vertices, indices);
        Ok(())
    })?;
    eprintln!("Culled {} occluded bricks", culled);

    // Rotation happens about the centroid, so the axis-aligned bounds grow.
    // Bound the rotated build by its corners rather than clipping it.
    let rotation = args.rotation.to_radians();
    let (x1, y1, x2, y2) = save.bounds;
    let (half_w, half_h) = rotated_half_extent(
        (x2 - x1) as f32 / 2.0,
        (y2 - y1) as f32 / 2.0,
        rotation,
    );

    let width = (half_w * 2.0 * args.scale).ceil() as u32 + args.margin * 2;
    let height = (half_h * 2.0 * args.scale).ceil() as u32 + args.margin * 2;

    // The view is centered on the centroid, but the bounds are not centered on
    // it in general — pan by the offset so the whole build lands in frame.
    let pan_x = -((x1 + x2) as f32 / 2.0 - save.centroid.0 as f32);
    let pan_y = -((y1 + y2) as f32 / 2.0 - save.centroid.1 as f32);

    // Anything past the device's max texture size is rendered as a grid of
    // tiles and stitched, so huge builds still produce one image
    let tile = renderer.max_tile_size();
    let cols = width.div_ceil(tile);
    let rows = height.div_ceil(tile);
    let tile_w = width.div_ceil(cols);
    let tile_h = height.div_ceil(rows);
    eprintln!("Rendering {}x{} px ({}x{} tiles of {}x{})", width, height, cols, rows, tile_w, tile_h);

    let mut stitcher = TileStitcher::default();
    stitcher.set_layout(tile_w, tile_h, rows, cols);

    // World-space size of one tile, used to walk the grid from its top-left
    let world_tile_w = tile_w as f32 / args.scale;
    let world_tile_h = tile_h as f32 / args.scale;
    let start_x = pan_x + (cols as f32 - 1.0) * world_tile_w / 2.0;
    let start_y = pan_y + (rows as f32 - 1.0) * world_tile_h / 2.0;

    for row in 0..rows {
        for col in 0..cols {
            let matrix = save.view_matrix(
                tile_w as f32,
                tile_h as f32,
                start_x - col as f32 * world_tile_w,
                start_y - row as f32 * world_tile_h,
                args.scale,
                rotation,
            );
            let pixels = renderer.render_to_pixels(tile_w, tile_h, &matrix)?.await?;
            stitcher.push_pixels(&pixels, row, col)?;
        }
    }

    stitcher.encode_png()
}

// Half-extent of an axis-aligned box rotated about its center
fn rotated_half_extent(half_w: f32, half_h: f32, rotation: f32) -> (f32, f32) {
    let (sin, cos) = rotation.sin_cos();
    (
        half_w * cos.abs() + half_h * sin.abs(),
        half_w * sin.abs() + half_h * cos.abs(),
    )
}
