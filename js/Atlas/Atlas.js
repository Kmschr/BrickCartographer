import React, {Component} from 'react';
import {saveBlob} from "./util";
import {ROTATE_CW, ROTATE_CCW, HOME, FULLSCREEN, BORDERS, FILL, PHOTO, MAP, LOAD} from "./icons";

import ACM_City from "../../default_saves/ACM_City.brs";

const ROTATE_ANGLE = Math.PI / 8;

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
        this.state = {
            save: null,
            fullscreen: false,
            showOutlines: false,
            fillBricks: true,
            rotation: 0,
            scale: 0.5,
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
                <div className="button-panel"></div>
                <div className="button-label view-label">VIEW</div>
                <div className="map-button zoom-in-button" title="Zoom In" onClick={() => this.zoomIn()}>+</div>
                <div className="map-button zoom-out-button" title="Zoom Out" onClick={() => this.zoomOut()}>-</div>
                <div className="map-button rotate-cw-button svg-button" title="Rotate CW" onClick={() => this.rotateCW()}>{ROTATE_CW}</div>
                <div className="map-button rotate-ccw-button svg-button" title="Rotate CCW" onClick={() => this.rotateCCW()}>{ROTATE_CCW}</div>
                <div className="map-button home-button svg-button" title="Home Position" onClick={() => this.resetPan()}>{HOME}</div>
                <div className="map-button fullscreen-button svg-button" title="Toggle Fullscreen" onClick={this.toggleFullscreen}>{FULLSCREEN}</div>
                <div className="button-label fill-label">FILL</div>
                <div className="map-button border-button svg-button" title="Toggle Brick Borders" onClick={this.toggleBrickOutlines}>{BORDERS}</div>
                <div className="map-button fill-button svg-button" title="Toggle Brick Fill" onClick={this.toggleBrickFill}>{FILL}</div>
                <div className="button-label save-label">SAVE</div>
                <div className="map-button photo-button svg-button" title="Save Current View" onClick={this.takeScreenshot}>{PHOTO}</div>
                <div className="map-button hd-photo-button svg-button" title="Save Entire Map (WIP)" onClick={this.takeHDScreenshot}>{MAP}</div>
                <div className="button-label load-label">LOAD</div>
                <div className="map-button load-button svg-button" title="Load Build" onClick={this.clickFileInput}>{LOAD}</div>
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
        let scrollDir = scrollEvent.deltaY;
        if (scrollDir > 0) {
            this.zoomOut();
        } else {
            this.zoomIn();
        }
    }

    zoomIn() {
        this.state.scale *= 2;
        this.redraw();
    }
    
    zoomOut() {
        this.state.scale /= 2;
        this.redraw();
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
                showOutlines: !this.state.showOutlines
            }, () => {
                this.processSave();
            });
        }
    }

    toggleBrickFill() {
        if (this.state.save) {
            this.setState({
                fillBricks: !this.state.fillBricks
            }, () => {
                this.processSave();
            })
        }
    }

    getNewPan(panStart, panEnd) {
        // get amount of panning occured
        let panDiff = {
            x: panEnd.x - panStart.x,
            y: panEnd.y - panStart.y
        };
        // Rotate the panning
        let diffRotated = {
            x: panDiff.x * Math.cos(this.state.rotation) - panDiff.y * Math.sin(this.state.rotation),
            y: panDiff.x * Math.sin(this.state.rotation) + panDiff.y * Math.cos(this.state.rotation)
        };
        // scale the amount of panning
        let diffScaled = {
            x: diffRotated.x / this.state.scale,
            y: diffRotated.y / this.state.scale
        };
        // apply the scaled amount of panning to original pre pan
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
        this.setState({
            loading: true
        }, () => {
                try {
                    this.state.save.buildVertexBuffer(this.state.showOutlines, this.state.fillBricks);
                } catch (err) {
                    console.error(err);
                }
                if (resetPan)
                    this.resetPan();
                this.canvas.style.cursor = null;
                this.redraw();
            }
        );
    }

}
