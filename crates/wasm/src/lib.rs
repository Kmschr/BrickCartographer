use brick_cartographer_core::save::{GeometryMode, GeometryState, SaveData, SaveLoading};
use brick_cartographer_core::{Renderer, TileStitcher};
use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> JsValue {
    JsValue::from(VERSION)
}

fn map_canvas() -> Result<web_sys::HtmlCanvasElement, JsValue> {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_elements_by_class_name("map-canvas").get_with_index(0))
        .ok_or_else(|| JsValue::from("Unable to find element with map-canvas class"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from("Unable to find HtmlCanvasElement"))
}

#[wasm_bindgen]
pub struct BRSProcessor {
    // Some while chunks are still streaming in; None once complete
    loading: Option<SaveLoading>,
    // Some once loading finished
    save: Option<SaveData>,
    geometry: GeometryState,
    renderer: Renderer,
}

/// Opens a save and prepares the renderer. Bricks stream in through
/// `loadStep`, which the frontend drives so it can repaint between steps.
#[wasm_bindgen(js_name = loadFile)]
pub async fn load_file(body: Vec<u8>) -> Result<BRSProcessor, JsValue> {
    console_error_panic_hook::set_once();

    let loading = SaveLoading::open(&body).map_err(JsValue::from)?;

    let canvas = map_canvas()?;
    let renderer = Renderer::new(Some(wgpu::SurfaceTarget::Canvas(canvas)))
        .await
        .map_err(JsValue::from)?;

    let geometry = GeometryState::new(loading.save(), GeometryMode::Map { outlines: false, fills: true });

    Ok(BRSProcessor {
        loading: Some(loading),
        save: None,
        geometry,
        renderer,
    })
}

#[wasm_bindgen]
impl BRSProcessor {
    fn save_ref(&self) -> &SaveData {
        match &self.loading {
            Some(loading) => loading.save(),
            None => self.save.as_ref().unwrap(),
        }
    }

    /// Parses and uploads chunks for roughly `budget_ms`, then returns
    /// loading progress in 0.0..=1.0 (1.0 = complete). The caller repaints
    /// and calls again until complete.
    #[wasm_bindgen(js_name = loadStep)]
    pub fn load_step(&mut self, budget_ms: f64) -> Result<f64, JsValue> {
        if self.loading.is_none() {
            return Ok(1.0);
        }

        let start = js_sys::Date::now();
        let mut done = false;
        {
            let loading = self.loading.as_mut().unwrap();
            loop {
                let more = loading.step().map_err(JsValue::from)?;
                self.geometry
                    .build_pending(loading.save(), &mut self.renderer)
                    .map_err(JsValue::from)?;
                if !more {
                    done = true;
                    break;
                }
                if js_sys::Date::now() - start >= budget_ms {
                    break;
                }
            }
        }
        // Push the partial batch so it reaches this repaint
        self.geometry.flush(&mut self.renderer);

        if done {
            let save = self.loading.take().unwrap().finish().map_err(JsValue::from)?;
            // A heightmap scales to the save's height extent, which only now
            // covers every chunk
            if self.geometry.mode() == GeometryMode::Heightmap {
                self.geometry = build_all(&save, GeometryMode::Heightmap, &mut self.renderer)?;
            }
            log(&format!("Bricks Discarded: {}", save.discarded));
            log(&format!("Bricks Culled: {}", self.geometry.culled));
            self.save = Some(save);
            return Ok(1.0);
        }
        Ok(self.loading.as_ref().unwrap().progress() as f64)
    }

    /// Switches what the map shows; rebuilds geometry for everything loaded
    /// so far. Streaming continues in the new mode.
    #[wasm_bindgen(js_name = setViewMode)]
    pub fn set_view_mode(&mut self, outlines: bool, fills: bool, heightmap: bool) -> Result<(), JsValue> {
        let mode = if heightmap {
            GeometryMode::Heightmap
        } else {
            GeometryMode::Map { outlines, fills }
        };
        let save = match &self.loading {
            Some(loading) => loading.save(),
            None => self.save.as_ref().unwrap(),
        };
        self.geometry = build_all(save, mode, &mut self.renderer)?;
        if self.loading.is_none() {
            log(&format!("Bricks Culled: {}", self.geometry.culled));
        }
        Ok(())
    }

    // Save info getters for frontend
    pub fn description(&self) -> String {
        self.save_ref().description.clone()
    }
    #[wasm_bindgen(js_name = brickCount)]
    pub fn brick_count(&self) -> i32 {
        self.save_ref().brick_count
    }
    pub fn centroid(&self) -> Array {
        let centroid = Array::new();
        centroid.push(&JsValue::from(self.save_ref().centroid.0));
        centroid.push(&JsValue::from(self.save_ref().centroid.1));
        centroid
    }
    pub fn bounds(&self) -> Array {
        let b = self.save_ref().bounds;
        let bounds = Array::new();
        bounds.push(&JsValue::from(b.0));
        bounds.push(&JsValue::from(b.1));
        bounds.push(&JsValue::from(b.2));
        bounds.push(&JsValue::from(b.3));
        bounds
    }

    pub fn render(&mut self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> Result<(), JsValue> {
        if size_x <= 0 || size_y <= 0 {
            return Ok(());
        }
        let matrix = self.save_ref().view_matrix(size_x as f32, size_y as f32, pan_x, pan_y, scale, rotation);
        self.renderer
            .render_to_surface(size_x as u32, size_y as u32, &matrix)
            .map_err(JsValue::from)
    }

    // Renders offscreen at the given size and resolves to a Promise of
    // tightly-packed RGBA pixels. Used for screenshot tiles.
    #[wasm_bindgen(js_name = renderToPixels)]
    pub fn render_to_pixels(&self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> Result<js_sys::Promise, JsValue> {
        let pixels = self.render_offscreen(size_x, size_y, pan_x, pan_y, scale, rotation)?;
        Ok(future_to_promise(async move {
            let pixels = pixels.await.map_err(JsValue::from)?;
            Ok(js_sys::Uint8Array::from(pixels.as_slice()).into())
        }))
    }

    // Like renderToPixels, but resolves to an encoded PNG
    #[wasm_bindgen(js_name = renderToPng)]
    pub fn render_to_png(&self, size_x: i32, size_y: i32, pan_x: f32, pan_y: f32, scale: f32, rotation: f32) -> Result<js_sys::Promise, JsValue> {
        let pixels = self.render_offscreen(size_x, size_y, pan_x, pan_y, scale, rotation)?;
        let (width, height) = (size_x as u32, size_y as u32);
        Ok(future_to_promise(async move {
            let pixels = pixels.await.map_err(JsValue::from)?;
            let png = brick_cartographer_core::encode_png(&pixels, width, height)
                .map_err(JsValue::from)?;
            Ok(js_sys::Uint8Array::from(png.as_slice()).into())
        }))
    }

    fn render_offscreen(
        &self,
        size_x: i32,
        size_y: i32,
        pan_x: f32,
        pan_y: f32,
        scale: f32,
        rotation: f32,
    ) -> Result<impl std::future::Future<Output = Result<Vec<u8>, String>> + use<>, JsValue> {
        if size_x <= 0 || size_y <= 0 {
            return Err(JsValue::from("invalid render size"));
        }
        let matrix = self.save_ref().view_matrix(size_x as f32, size_y as f32, pan_x, pan_y, scale, rotation);
        self.renderer
            .render_to_pixels(size_x as u32, size_y as u32, &matrix)
            .map_err(JsValue::from)
    }
}

fn build_all(save: &SaveData, mode: GeometryMode, renderer: &mut Renderer) -> Result<GeometryState, JsValue> {
    renderer.clear_batches();
    let mut state = GeometryState::new(save, mode);
    state.build_pending(save, renderer).map_err(JsValue::from)?;
    state.flush(renderer);
    Ok(state)
}

#[wasm_bindgen]
pub struct ImageCombiner {
    stitcher: TileStitcher,
}

#[wasm_bindgen(js_name = getImageCombiner)]
pub fn get_image_combiner() -> ImageCombiner {
    console_error_panic_hook::set_once();
    ImageCombiner { stitcher: TileStitcher::default() }
}

#[wasm_bindgen]
impl ImageCombiner {
    #[wasm_bindgen(js_name = setLayout)]
    pub fn set_layout(&mut self, tile_width: u32, tile_height: u32, rows: u32, cols: u32) {
        self.stitcher.set_layout(tile_width, tile_height, rows, cols);
    }

    #[wasm_bindgen(js_name = pushPixels)]
    pub fn push_pixels(&mut self, tile: &[u8], row: u32, col: u32) -> Result<(), JsValue> {
        self.stitcher.push_pixels(tile, row, col).map_err(JsValue::from)
    }

    #[wasm_bindgen(js_name = combineImages)]
    pub fn combine_images(&mut self) -> Result<Vec<u8>, JsValue> {
        self.stitcher.encode_png().map_err(JsValue::from)
    }
}
