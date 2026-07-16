import { saveBlob } from "./util";
import { ROTATE_CW, ROTATE_CCW, HOME, FULLSCREEN, BORDERS, FILL, MOUNTAIN, PHOTO, MAP, LEGO, LOAD, GITHUB } from "./icons";
import ACM_City from "../../default_saves/ACM_City.brs";
import wasm from "../wasm";

const DEFAULT_ROTATION = 0;
const ROTATE_ANGLE = Math.PI / 8;
const DEFAULT_SCALE = 0.1;
const MAX_SCALE = 20;
const MIN_SCALE = 0.01;
const DEFAULT_PAN = { x: 0, y: 0 };
const SCROLL_INTENSITY = 1.2;
const WEBGPU_NOTICE_DISMISSED = "webgpu-notice-dismissed";

// navigator.gpu can exist while no adapter is actually obtainable, which is
// why the renderer probes for a real adapter rather than trusting the object
async function webGPUAvailable() {
    if (!navigator.gpu) return false;
    try {
        return !!(await navigator.gpu.requestAdapter());
    } catch {
        return false;
    }
}

export default class Atlas {
    constructor(root) {
        this.save = null;
        this.map = "";
        this.loading = false;
        this.fullscreen = false;
        this.showOutlines = false;
        this.fillBricks = true;
        this.showHeightmap = false;
        this.isDragging = false;
        this.rotation = DEFAULT_ROTATION;
        this.scale = DEFAULT_SCALE;
        this.pan = DEFAULT_PAN;
        this.dragPos = DEFAULT_PAN;

        this.buildDom(root);
        this.attachEvents();
        this.checkWebGPU();

        wasm.then(rust => console.log("v" + rust.getVersion()));
        wasm.then(rust => rust.getImageCombiner()).then(ic => this.imageCombiner = ic);

        window.onresize = () => this.redraw();
        this.loadDefaultCity();
    }

    buildDom(root) {
        root.innerHTML = `
            <div id="ui-container">
                <div id="map-container" class="map-container">
                    <canvas class="map-canvas draggable"></canvas>
                </div>
                <div class="button-panel"></div>
                <div class="button-label view-label">VIEW</div>
                <div class="map-button zoom-in-button" title="Zoom In">+</div>
                <div class="map-button zoom-out-button" title="Zoom Out">-</div>
                <div class="map-button rotate-cw-button svg-button" title="Rotate CW">${ROTATE_CW}</div>
                <div class="map-button rotate-ccw-button svg-button" title="Rotate CCW">${ROTATE_CCW}</div>
                <div class="map-button home-button svg-button" title="Home Position">${HOME}</div>
                <div class="map-button fullscreen-button svg-button" title="Toggle Fullscreen">${FULLSCREEN}</div>
                <div class="button-label fill-label">FILL</div>
                <div class="map-button border-button svg-button" title="Toggle Brick Borders">${BORDERS}</div>
                <div class="map-button fill-button svg-button button-toggled" title="Toggle Brick Fill">${FILL}</div>
                <div class="map-button heightmap-button svg-button" title="Toggle Heightmap">${MOUNTAIN}</div>
                <div class="button-label save-label">SAVE</div>
                <div class="map-button photo-button svg-button" title="Save Current View">${PHOTO}</div>
                <div class="map-button hd-photo-button svg-button" title="Save Entire Map">${MAP}</div>
                <div class="map-button zoom-photo-button svg-button" title="Save Entire Map x10 Zoom">${LEGO}</div>
                <div class="button-label load-label">LOAD</div>
                <div class="map-button load-button svg-button" title="Load Build">${LOAD}</div>
                <a class="github-button" href="https://github.com/Kmschr/BrickCartographer" target="_blank" rel="noopener noreferrer">${GITHUB}</a>
                <input type="file" name="file" id="file" />
                <div class="loading-overlay" style="display:none"><div class="loading-spinner"></div></div>
                <div class="webgpu-notice" style="display:none">
                    <span class="webgpu-notice-text"></span>
                    <button class="webgpu-notice-dismiss" title="Dismiss">&times;</button>
                </div>
            </div>`;

        const $ = sel => root.querySelector(sel);
        this.canvas = $(".map-canvas");
        this.fileInput = $("#file");
        this.loadingOverlay = $(".loading-overlay");
        this.webgpuNotice = $(".webgpu-notice");
        this.borderButton = $(".border-button");
        this.fillButton = $(".fill-button");
        this.heightmapButton = $(".heightmap-button");
        this.el = { $ };
    }

    attachEvents() {
        const $ = this.el.$;

        $("#map-container").addEventListener("dragover", e => { e.preventDefault(); e.dataTransfer.dropEffect = "move"; });
        this.canvas.addEventListener("mousedown", e => this.handleMouseDownEvent(e));
        this.canvas.addEventListener("mousemove", e => this.handleMouseMoveEvent(e));
        this.canvas.addEventListener("mouseup", () => this.handleMouseUpEvent());
        this.canvas.addEventListener("touchstart", e => this.handleMouseDownEvent(e));
        this.canvas.addEventListener("touchmove", e => this.handleMouseMoveEvent(e));
        this.canvas.addEventListener("touchend", () => this.handleMouseUpEvent());
        this.canvas.addEventListener("touchcancel", () => this.handleMouseUpEvent());
        this.canvas.addEventListener("wheel", e => this.handleWheelEvent(e));

        $(".button-panel").addEventListener("mousemove", () => this.isDragging = false);
        $(".zoom-in-button").addEventListener("click", () => { this.zoomIn(); this.redraw(); });
        $(".zoom-out-button").addEventListener("click", () => { this.zoomOut(); this.redraw(); });
        $(".rotate-cw-button").addEventListener("click", () => this.rotateCW());
        $(".rotate-ccw-button").addEventListener("click", () => this.rotateCCW());
        $(".home-button").addEventListener("click", () => this.resetView());
        $(".fullscreen-button").addEventListener("click", () => this.toggleFullscreen());
        this.borderButton.addEventListener("click", () => this.toggleBrickOutlines());
        this.fillButton.addEventListener("click", () => this.toggleBrickFill());
        this.heightmapButton.addEventListener("click", () => this.toggleHeightmap());
        $(".photo-button").addEventListener("click", () => this.takeScreenshot());
        $(".hd-photo-button").addEventListener("click", () => this.takeHDScreenshot(1));
        $(".zoom-photo-button").addEventListener("click", () => this.takeHDScreenshot(10));
        $(".load-button").addEventListener("click", () => this.clickFileInput());
        this.fileInput.addEventListener("change", e => this.handleFileSelected(e));
        $(".webgpu-notice-dismiss").addEventListener("click", () => this.dismissWebGPUNotice());
    }

    // The renderer falls back to WebGL2 wherever WebGPU is missing, so the map
    // still works — this only tells Firefox users the faster path exists, since
    // Firefox still ships WebGPU disabled by default outside Windows.
    async checkWebGPU() {
        if (localStorage.getItem(WEBGPU_NOTICE_DISMISSED)) return;
        if (await webGPUAvailable()) return;
        if (!navigator.userAgent.includes("Firefox")) return;

        this.webgpuNotice.querySelector(".webgpu-notice-text").innerHTML =
            'Running on WebGL. For faster rendering, enable WebGPU in Firefox: open ' +
            '<code>about:config</code> and set <code>dom.webgpu.enabled</code> to <code>true</code>.';
        this.webgpuNotice.style.display = "flex";
    }

    dismissWebGPUNotice() {
        this.webgpuNotice.style.display = "none";
        localStorage.setItem(WEBGPU_NOTICE_DISMISSED, "1");
    }

    setLoading(loading) {
        this.loading = loading;
        this.loadingOverlay.style.display = loading ? "flex" : "none";
    }

    syncToggleButtons() {
        this.borderButton.classList.toggle("button-toggled", this.showOutlines);
        this.fillButton.classList.toggle("button-toggled", this.fillBricks);
        this.heightmapButton.classList.toggle("button-toggled", this.showHeightmap);
    }

    // Flip loading on, yield a frame so the spinner paints, then run the
    // (blocking) wasm work and clear loading regardless of outcome.
    withLoading(task) {
        this.setLoading(true);
        requestAnimationFrame(() => setTimeout(() => {
            Promise.resolve().then(task).finally(() => this.setLoading(false));
        }, 0));
    }

    redraw() {
        if (!this.save) return;
        if (this.canvas.width != this.canvas.clientWidth || this.canvas.height != this.canvas.clientHeight) {
            this.canvas.width = this.canvas.clientWidth;
            this.canvas.height = this.canvas.clientHeight;
        }
        this.save.render(this.canvas.width, this.canvas.height, this.pan.x, this.pan.y, this.scale, this.rotation);
    }

    handleMouseDownEvent(event) {
        event = this.handleTouchEvent(event);
        this.dragPos = { x: event.clientX, y: event.clientY };
        this.isDragging = true;
    }

    handleMouseMoveEvent(event) {
        if (!this.isDragging) return;
        event = this.handleTouchEvent(event);
        let newDragPos = { x: event.clientX, y: event.clientY };
        this.pan = this.getNewPan(this.dragPos, newDragPos);
        this.dragPos = newDragPos;
        this.redraw();
    }

    handleMouseUpEvent() {
        this.isDragging = false;
    }

    handleTouchEvent(event) {
        if (!event.clientX) return event.touches[0];
        return event;
    }

    handleWheelEvent(event) {
        let mousePos = { x: event.clientX, y: event.clientY };
        let centerPos = { x: this.canvas.width / 2, y: this.canvas.height / 2 };
        this.pan = this.getNewPan(mousePos, centerPos);
        if (event.deltaY > 0) {
            this.zoomOut();
        } else {
            this.zoomIn();
        }
        this.pan = this.getNewPan(centerPos, mousePos);
        this.redraw();
    }

    zoomIn() {
        if (this.scale < MAX_SCALE) this.scale *= SCROLL_INTENSITY;
    }

    zoomOut() {
        if (this.scale > MIN_SCALE) this.scale /= SCROLL_INTENSITY;
    }

    loadDefaultCity() {
        this.withLoading(() =>
            fetch(ACM_City)
                .then(res => res.arrayBuffer())
                .then(buff => wasm.then(rust => rust.loadFile(new Uint8Array(buff))))
                .then(save => {
                    this.replaceSave(save);
                    this.map = "ACM City";
                    this.processSave(true);
                })
                .catch(err => console.error(err))
        );
    }

    toggleFullscreen() {
        if (!this.fullscreen) {
            document.getElementById("ui-container").requestFullscreen();
        } else {
            document.exitFullscreen();
        }
        this.fullscreen = !this.fullscreen;
    }

    takeScreenshot() {
        if (!this.save) return;
        this.save.renderToPng(this.canvas.width, this.canvas.height, this.pan.x, this.pan.y, this.scale, this.rotation)
            .then(png => saveBlob(new Blob([png.buffer], { type: "image/png" }), `${this.map}.png`))
            .catch(err => console.error(err));
    }

    // Render the whole build as a grid of viewport-sized tiles, offscreen, and
    // stitch them in wasm. Tiles come back as raw RGBA and the combiner holds
    // the full-size buffer in wasm memory and encodes the PNG there, so the
    // output isn't bounded by the browser's max 2d-canvas size — large builds
    // (Orion's Freebuild etc.) would otherwise exceed it and produce no image
    // at all. Tiles render sequentially to bound peak memory.
    async takeHDScreenshot(zoom) {
        if (!this.save) return;

        const scale = DEFAULT_SCALE * zoom;
        const tileWidth = this.canvas.width;
        const tileHeight = this.canvas.height;

        const bounds = this.save.bounds();
        const worldTileWidth = tileWidth / scale;
        const worldTileHeight = tileHeight / scale;
        bounds[0] += worldTileWidth / 2;
        bounds[1] += worldTileHeight / 2;
        const imageWidth = bounds[2] - bounds[0];
        const imageHeight = bounds[3] - bounds[1];
        const numCols = Math.max(1, Math.ceil(imageWidth / worldTileWidth));
        const numRows = Math.max(1, Math.ceil(imageHeight / worldTileHeight));

        try {
            this.imageCombiner.setLayout(tileWidth, tileHeight, numRows, numCols);
            for (let col = 0; col < numCols; col++) {
                for (let row = 0; row < numRows; row++) {
                    const pixels = await this.save.renderToPixels(
                        tileWidth, tileHeight,
                        -bounds[0] - col * worldTileWidth, -bounds[1] - row * worldTileHeight,
                        scale, DEFAULT_ROTATION);
                    this.imageCombiner.pushPixels(pixels, row, col);
                }
            }
            const buffer = this.imageCombiner.combineImages();
            saveBlob(new Blob([buffer.buffer], { type: "image/png" }), `${this.map}.png`);
        } catch (err) {
            console.error(err);
        }
    }

    rotateCCW() {
        this.rotation += ROTATE_ANGLE;
        this.redraw();
    }

    rotateCW() {
        this.rotation -= ROTATE_ANGLE;
        this.redraw();
    }

    toggleBrickOutlines() {
        if (!this.save) return;
        this.showOutlines = !this.showOutlines;
        this.showHeightmap = false;
        this.syncToggleButtons();
        this.withLoading(() => this.processSave());
    }

    toggleBrickFill() {
        if (!this.save) return;
        this.fillBricks = !this.fillBricks;
        this.showHeightmap = false;
        this.syncToggleButtons();
        this.withLoading(() => this.processSave());
    }

    toggleHeightmap() {
        if (!this.save) return;
        this.showHeightmap = !this.showHeightmap;
        this.fillBricks = false;
        this.showOutlines = false;
        this.syncToggleButtons();
        this.withLoading(() => this.processSave());
    }

    getNewPan(panStart, panEnd) {
        let panDiff = {
            x: panEnd.x - panStart.x,
            y: panEnd.y - panStart.y
        };
        let diffRotated = {
            x: panDiff.x * Math.cos(this.rotation) - panDiff.y * Math.sin(this.rotation),
            y: panDiff.x * Math.sin(this.rotation) + panDiff.y * Math.cos(this.rotation)
        };
        return {
            x: this.pan.x + diffRotated.x / this.scale,
            y: this.pan.y + diffRotated.y / this.scale
        };
    }

    resetView() {
        this.scale = DEFAULT_SCALE;
        this.pan = DEFAULT_PAN;
        this.rotation = DEFAULT_ROTATION;
        this.redraw();
    }

    // Frees the old save's GPU buffers and device promptly instead of
    // waiting on wasm-bindgen's GC finalizer
    replaceSave(save) {
        if (this.save) this.save.free();
        this.save = save;
    }

    clickFileInput() {
        this.fileInput.click();
    }

    handleFileSelected(event) {
        let file = event.target.files[0];
        if (file) this.withLoading(() => this.loadFileWASM(file));
    }

    loadFileWASM(file) {
        const filename = file.name.replace(/\.[^/.]+$/, "");
        return file.arrayBuffer()
            .then(buff => wasm.then(rust => rust.loadFile(new Uint8Array(buff))))
            .then(save => {
                this.replaceSave(save);
                this.map = filename;
                this.processSave(true);
            })
            .catch(err => console.error(err));
    }

    processSave(newSave) {
        try {
            if (this.showHeightmap) {
                this.save.buildHeightmapVertexBuffer();
            } else {
                this.save.buildVertexBuffer(this.showOutlines, this.fillBricks);
            }
        } catch (err) {
            console.error(err);
        }
        if (newSave) this.resetView();
        this.canvas.style.cursor = null;
        this.redraw();
    }
}
