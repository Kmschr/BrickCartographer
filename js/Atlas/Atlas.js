import React, {Component} from 'react';
import {saveBlob} from "./util";
import {ROTATE_CW, ROTATE_CCW, HOME, FULLSCREEN, BORDERS, FILL, MOUNTAIN, PHOTO, MAP, LOAD, GITHUB} from "./icons";

import ACM_City from "../../default_saves/ACM_City.brs";

const DEFAULT_ROTATION = 0;
const ROTATE_ANGLE = Math.PI / 8;
const DEFAULT_SCALE = 0.1;
const MAX_SCALE = 20;
const MIN_SCALE = 0.01;
const SCROLL_INTENSITY = 1.2;

const wasm = import('../../pkg');

export default class Atlas extends Component {

    constructor(props) {
        super(props);
        this.loadFileWASM = this.loadFileWASM.bind(this);
        this.redraw = this.redraw.bind(this);
        this.resetPan = this.resetPan.bind(this);
        this.processSave = this.processSave.bind(this);
        this.toggleFullscreen = this.toggleFullscreen.bind(this);
        this.takeScreenshot = this.takeScreenshot.bind(this);
        this.takeHDScreenshot = this.takeHDScreenshot.bind(this);
        this.toggleBrickOutlines = this.toggleBrickOutlines.bind(this);
        this.loadDefaultCity = this.loadDefaultCity.bind(this);
        this.toggleBrickFill = this.toggleBrickFill.bind(this);
        this.toggleHeightmap = this.toggleHeightmap.bind(this);
        this.state = {
            save: null,
            fullscreen: false,
            showOutlines: false,
            fillBricks: true,
            showHeightmap: false,
            rotation: DEFAULT_ROTATION,
            scale: DEFAULT_SCALE,
            pan: {
                x: 0,
                y: 0
            },
            dragPos: {
                x: 0,
                y: 0
            }
        };
    }

    render() {
        let borderButtonClassName = "map-button border-button svg-button";
        let fillButtonClassName = "map-button fill-button svg-button";
        let heightmapButtonClassName = "map-button heightmap-button svg-button";
        if (this.state.showOutlines) {
            borderButtonClassName += " button-toggled";
        }
        if (this.state.fillBricks) {
            fillButtonClassName += " button-toggled";
        }
        if (this.state.showHeightmap) {
            heightmapButtonClassName += " button-toggled";
        }

        return (
            <div id="ui-container">
                <div id="map-container" className="map-container" onDragOver={(e) => {e.preventDefault(); e.dataTransfer.dropEffect="move"}}>
                    <canvas 
                        ref={(ref) => this.canvas=ref} 
                        className={"map-canvas draggable"}
                        onMouseDown={(e) => this.handleMouseDownEvent(e)}
                        onMouseMove={(e) => this.handleMouseMoveEvent(e)}
                        onMouseUp={() => this.handleMouseUpEvent()}
                        onWheel={(e) => this.handleWheelEvent(e)}
                    />
                </div>
                <div className="button-panel" onMouseMove={() => this.state.isDragging = false}></div>
                <div className="button-label view-label">VIEW</div>
                <div className="map-button zoom-in-button" title="Zoom In" onClick={() => {this.zoomIn(); this.redraw()}}>+</div>
                <div className="map-button zoom-out-button" title="Zoom Out" onClick={() => {this.zoomOut(); this.redraw()}}>-</div>
                <div className="map-button rotate-cw-button svg-button" title="Rotate CW" onClick={() => this.rotateCW()}>{ROTATE_CW}</div>
                <div className="map-button rotate-ccw-button svg-button" title="Rotate CCW" onClick={() => this.rotateCCW()}>{ROTATE_CCW}</div>
                <div className="map-button home-button svg-button" title="Home Position" onClick={() => this.resetPan()}>{HOME}</div>
                <div className="map-button fullscreen-button svg-button" title="Toggle Fullscreen" onClick={this.toggleFullscreen}>{FULLSCREEN}</div>
                <div className="button-label fill-label">FILL</div>
                <div className={borderButtonClassName} title="Toggle Brick Borders" onClick={this.toggleBrickOutlines}>{BORDERS}</div>
                <div className={fillButtonClassName} title="Toggle Brick Fill" onClick={this.toggleBrickFill}>{FILL}</div>
                <div className={heightmapButtonClassName} title="Toggle Heightmap" onClick={this.toggleHeightmap}>{MOUNTAIN}</div>
                <div className="button-label save-label">SAVE</div>
                <div className="map-button photo-button svg-button" title="Save Current View" onClick={this.takeScreenshot}>{PHOTO}</div>
                <div className="map-button hd-photo-button svg-button" title="Save Entire Map (WIP)" onClick={this.takeHDScreenshot}>{MAP}</div>
                <div className="button-label load-label">LOAD</div>
                <div className="map-button load-button svg-button" title="Load Build" onClick={this.clickFileInput}>{LOAD}</div>
                <a className="github-button" href="https://github.com/Kmschr/BrickCartographer" target="_blank" rel="noopener noreferrer">{GITHUB}</a>
                <input type='file' name='file' id='file' onChange={(e) => this.handleFileSelected(e)}/>
                { /*<SaveInfo map={this.state.map} save={this.state.save} /> */ }
            </div>
        )
    }

    redraw() {
        if (this.state.save) {
            if (this.canvas.width != this.canvas.clientWidth || this.canvas.height != this.canvas.clientHeight) {
                this.canvas.width = this.canvas.clientWidth;
                this.canvas.height = this.canvas.clientHeight;
            }
            this.state.save.render(this.canvas.width, this.canvas.height, this.state.pan.x, this.state.pan.y, this.state.scale, this.state.rotation);
        }
    }

    handleMouseDownEvent(event) {
        event.persist();
        this.state.dragPos = {
            x: event.clientX,
            y: event.clientY
        }
        this.state.isDragging = true;
    }

    handleMouseMoveEvent(event) {
        if (this.state.isDragging) {
            event.persist();
            let newDragPos = {
                x: event.clientX,
                y: event.clientY
            }
            let newPan = this.getNewPan(this.state.dragPos, newDragPos);
            this.state.dragPos = newDragPos;
            this.state.pan = newPan;
            this.redraw();
        }
    }

    handleMouseUpEvent() {
        this.state.isDragging = false;
    }

    handleWheelEvent(event) {
        let scrollEvent = event;
        let mousePos = {
            x: event.clientX,
            y: event.clientY
        };
        let centerPos = {
            x: this.canvas.width / 2,
            y: this.canvas.height / 2
        };
        let tempPan = this.getNewPan(mousePos, centerPos);
        this.state.pan = tempPan;
        let scrollDir = scrollEvent.deltaY;
        if (scrollDir > 0) {
            this.zoomOut();
        } else {
            this.zoomIn();
        }
        let finalPan = this.getNewPan(centerPos, mousePos);
        this.state.pan = finalPan;
        this.redraw();
    }

    zoomIn() {
        if (this.state.scale < MAX_SCALE) {
            this.state.scale *= SCROLL_INTENSITY;
        }
    }
    
    zoomOut() {
        if (this.state.scale > MIN_SCALE) {
            this.state.scale /= SCROLL_INTENSITY;
        }
    }

    componentDidMount() {
        wasm.then(rust => rust.getImageCombiner()).then(
            ic => {
                this.imageCombiner = ic;
            }
        );
        window.onresize = () => {
            this.redraw()
        }
        this.loadDefaultCity();
    }

    loadDefaultCity() {
        let xhr = new XMLHttpRequest();
        xhr.open("GET", ACM_City);
        xhr.responseType = "arraybuffer";
        xhr.onreadystatechange = _ => {
            if (xhr.readyState === XMLHttpRequest.DONE) {
                if (xhr.status === 200) {
                    let buff = xhr.response;                    
                    let u8buff = new Uint8Array(buff);
                    wasm.then(rust => rust.loadFile(u8buff)).catch((error) => {
                            this.setState({
                                fileError: true,
                                fileErrorMsg: error
                            });
                        })
                        .then(save => {                            
                            this.setState({
                                save: save,
                                map: "ACM City"
                            }, () => {
                                this.processSave(true);
                            });
                        });
                }  
            }
        }
        xhr.send();
    }

    toggleFullscreen() {
        if (!this.state.fullscreen) {
            let mapContainer = document.getElementById("ui-container");
            mapContainer.requestFullscreen()
        } else {
            document.exitFullscreen();
        }

        this.setState({
            fullscreen: !this.state.fullscreen
        })
    }

    takeScreenshot() {
        if (this.state.save) {
            this.state.save.render(this.canvas.width, this.canvas.height, this.state.pan.x, this.state.pan.y, this.state.scale, this.state.rotation);
            this.canvas.toBlob((blob) => {
                saveBlob(blob, `${this.state.map}.png`);
            });
        }
    }

    takeHDScreenshot() {
        if (this.state.save) {
            this.imageCombiner.setSize(this.canvas.width, this.canvas.height);
            let bounds = this.state.save.bounds();
            let canvasWidth = this.canvas.width / this.state.scale;
            let canvasHeight = this.canvas.height / this.state.scale;
            bounds[0] += canvasWidth / 2;
            bounds[1] += canvasHeight / 2;
            let imageWidth = (bounds[2] - bounds[0]);
            let imageHeight = (bounds[3] - bounds[1]);
            let numCols = Math.ceil(imageWidth / canvasWidth);
            let numRows = Math.ceil(imageHeight / canvasHeight);
            let numImages = numRows * numCols;
            let imageIndex = 0;
            for (let col = 0; col < numCols; col++) {
                for (let row=0; row < numRows; row++) {
                    let x = col * canvasWidth;
                    let y = row * canvasHeight;
                    this.setPan(-bounds[0] - x, -bounds[1] - y);
                    this.state.save.render(this.canvas.width, this.canvas.height, this.state.pan.x, this.state.pan.y, this.state.scale, this.state.rotation);
                    this.canvas.toBlob((blob) => {
                        blob.arrayBuffer().then(buff => {
                            let u8buff = new Uint8Array(buff);
                            this.imageCombiner.pushImage(u8buff, row*numCols + col);
                            imageIndex++;
                            if (imageIndex === numImages) {
                                try {
                                    let buffer = this.imageCombiner.combineImages(numRows, numCols);
                                    let merged = new Blob([buffer.buffer]);
                                    //console.log(merged);
                                    saveBlob(merged, `${this.state.map}.png`);
                                } catch (err) {
                                    console.log(err);
                                }
                            }
                        });
                    }, "image/png")
                }
            }
        }
    }

    setPan(x, y) {
        this.state.pan = {
            x: x,
            y: y
        };
    }

    rotateCCW() {
        this.state.rotation += ROTATE_ANGLE;
        this.redraw();
    }

    rotateCW() {
        this.state.rotation -= ROTATE_ANGLE;
        this.redraw();
    }

    toggleBrickOutlines() {
        if (this.state.save) {
            this.setState({
                showOutlines: !this.state.showOutlines,
                showHeightmap: false
            }, () => {
                this.processSave();
            });
        }
    }

    toggleBrickFill() {
        if (this.state.save) {
            this.setState({
                fillBricks: !this.state.fillBricks,
                showHeightmap: false,
            }, () => {
                this.processSave();
            })
        }
    }

    toggleHeightmap() {
        if (this.state.save) {
            this.setState({
                showHeightmap: !this.state.showHeightmap,
                fillBricks: false,
                showOutlines: false
            }, () => {
                this.processSave();
            })
        }
    }

    getNewPan(panStart, panEnd) {
        let panDiff  = {
            x: panEnd.x - panStart.x,
            y: panEnd.y - panStart.y
        };
        let diffRotated = {
            x: panDiff.x * Math.cos(this.state.rotation) - panDiff.y * Math.sin(this.state.rotation),
            y: panDiff.x * Math.sin(this.state.rotation) + panDiff.y * Math.cos(this.state.rotation)
        };
        let diffScaled = {
            x: diffRotated.x / this.state.scale,
            y: diffRotated.y / this.state.scale
        };
        return {
            x: this.state.pan.x + diffScaled.x,
            y: this.state.pan.y + diffScaled.y
        };
    }

    resetPan() {
        this.state.pan = {
            x: 0,
            y: 0
        };
        this.redraw();
    }

    clickFileInput() {
        var fileInput = document.getElementById('file');
        if(fileInput)
             fileInput.click();
    }

    handleFileSelected(event) {
        let file = event.target.files[0];
        if (file) {
            this.canvas.style.cursor = "wait";
            this.loadFileWASM(file);
        }
    }

    loadFileWASM(file) {
        file.arrayBuffer()
            .then(buff => new Uint8Array(buff))
            .then(buff =>
                wasm.then(rust => rust.loadFile(buff)).catch((error) => {
                    this.setState({
                        fileError: true,
                        fileErrorMsg: error
                    });
                })
            )
            .then(save => {
                this.setState({
                    save: save,
                }, () => {
                    this.processSave(true);
                });
            });
    }

    processSave(resetPan) {
        try {
            if (this.state.showHeightmap) {
                this.state.save.buildHeightmapVertexBuffer();
            } else {
                this.state.save.buildVertexBuffer(this.state.showOutlines, this.state.fillBricks);
            }
        } catch (err) {
            console.error(err);
        }
        if (resetPan)
            this.resetPan();
        this.canvas.style.cursor = null;
        this.redraw();

    }

}
