import React, {Component} from 'react';
import {Col, Container, Row, Input, Spinner, Alert, Button, CardLink} from 'reactstrap';
import {removeFileExtension} from "../util";
import 'leaflet/dist/leaflet.css';
import '../style.css';
import './L.CanvasLayer';
import 'leaflet-easybutton';
import SaveInfo from "./SaveInfo";
import {saveBlob} from "./util";

import ACM_City from "../../default_saves/ACM_City.brs";

const MAP_CENTER_DEFAULT = [0, 0];
const MAP_ZOOM_DEFAULT = 0;
const MAP_ZOOM_MIN = -6;

const DEG45 = Math.PI / 4;

const wasm = import('../../pkg');

export default class Atlas extends Component {

    constructor(props) {
        super(props);
        this.handleFileSelected = this.handleFileSelected.bind(this);
        this.loadFile = this.loadFile.bind(this);
        this.loadFileWASM = this.loadFileWASM.bind(this);
        this.onDrawLayer = this.onDrawLayer.bind(this);
        this.getNewPan = this.getNewPan.bind(this);
        this.resetPan = this.resetPan.bind(this);
        this.processSave = this.processSave.bind(this);
        this.toggleFullscreen = this.toggleFullscreen.bind(this);
        this.takeScreenshot = this.takeScreenshot.bind(this);
        this.toggleBrickOutlines = this.toggleBrickOutlines.bind(this);
        this.loadDefaultCity = this.loadDefaultCity.bind(this);
        this.toggleBrickFill = this.toggleBrickFill.bind(this);
        this.state = {
            fileError: false,
            fileErrorMsg: null,
            loading: false,
            map: null,
            save: null,
            fullscreen: false,
            showOutlines: false,
            fillBricks: true,
            rotation: 0,
        };

        this.resetPan();
    }

    componentDidMount() {
        // Initialize Map
        this.map = L.map('map', {
            crs: L.CRS.Simple,
            center: MAP_CENTER_DEFAULT,
            zoom: MAP_ZOOM_DEFAULT,
            minZoom: MAP_ZOOM_MIN,
            attributionControl: false,
            scrollWheelZoom: true,
            doubleClickZoom: false,
        });

        L.easyButton({
            position: 'topright',
            states: [{
                icon: '<i class="fas fa-expand map-button"></i>',
                title: 'Fullscreen',
                onClick: this.toggleFullscreen
            }]
        }).addTo(this.map);

        L.easyButton({
            position: 'bottomright',
            states: [{
                icon: '<i class="fas fa-camera map-button"></i>',
                title: 'Take Screenshot',
                onClick: this.takeScreenshot
            }]
        }).addTo(this.map);

        let CCW = L.easyButton({
            position: 'topleft',
            states: [{
                icon: '<i class="fas fa-undo map-button"></i>',
                title: 'Rotate CCW',
                onClick: this.rotateCCW.bind(this)
            }]
        });
        let CW = L.easyButton({
            position: 'topleft',
            states: [{
                icon: '<i class="fas fa-redo map-button"></i>',
                title: 'Rotate CW',
                onClick: this.rotateCCW.bind(this)
            }]
        });
        L.easyBar([CCW, CW], {
            position: 'topleft'
        }).addTo(this.map);

        L.easyButton({
            position: 'bottomleft',
            states: [{
                icon: '<i class="fas fa-file-upload map-button"></i>',
                title: 'Load Save',
                onClick: this.clickFileInput
            }]
        }).addTo(this.map);

        let outlines = L.easyButton({
            position: 'topleft',
            states: [{
                icon: '<i class="fas fa-border-style map-button"></i>',
                title: 'Toggle Brick Outlines',
                onClick: this.toggleBrickOutlines
            }]
        });
        let fill = L.easyButton({
            position: 'topleft',
            states: [{
                icon: '<i class="fas fa-fill map-button"></i>',
                title: 'Toggle Brick Fill',
                onClick: this.toggleBrickFill
            }]
          });
        L.easyBar([outlines, fill], {
            position: 'bottomleft',
        }).addTo(this.map);

        // Add a HTMLCanvas to the Map
        this.canvas = L.canvasLayer().delegate(this);
        this.canvas.addTo(this.map);

        this.loadDefaultCity();
    }

    loadDefaultCity() {
        let xhr = new XMLHttpRequest();
        xhr.open("GET", ACM_City);
        xhr.responseType = "arraybuffer";
        xhr.onreadystatechange = _ => {
            if(xhr.readyState === 4) {
                if(xhr.status === 200 || rawFile.status == 0) {
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
            let mapContainer = L.DomUtil.get("map-container");
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
            let scale = Math.pow(2, this.map.getZoom());
            this.state.save.render(this.canvas._canvas.width, this.canvas._canvas.height, this.state.pan.x, this.state.pan.y, scale, this.state.rotation);
            this.canvas._canvas.toBlob((blob) => {
                saveBlob(blob, `${this.state.map}.png`);
            });
        }
    }

    rotateCCW() {
        this.setState({
            rotation: this.state.rotation + DEG45
        }, () => {
            this.canvas.needRedraw();
        })
    }

    rotateCW() {
        this.setState({
            rotation: this.state.rotation - DEG45
        }, () => {
            this.canvas.needRedraw();
        })
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

    // Called upon any pan/zoom of map by canvas layer library
    onDrawLayer(info) {
        if (this.state.save) {
            // get current pan and current scale
            let pane = this.map._getMapPanePos();
            let scale = Math.pow(2, this.map.getZoom());

            let newPan = this.getNewPan(pane, scale);

            this.state.save.render(info.canvas.width, info.canvas.height, newPan.x, newPan.y, scale, this.state.rotation);

            this.state.pan = newPan;
            this.state.pane.x = pane.x;
            this.state.pane.y = pane.y;

        }
    }

    getNewPan(pane, scale) {
        if (!this.state.pane) {
            this.state.pane = {
                x: pane.x,
                y: pane.y
            }
        }
        // get amount of panning occured
        let panDiff = {
            x: pane.x - this.state.pane.x,
            y: pane.y - this.state.pane.y
        };
        // scale the amount of panning
        let diffScaled = {
            x: panDiff.x / scale,
            y: panDiff.y / scale
        };
        // apply the scaled amount of panning to original pre pan
        return {
            x: this.state.pan.x + diffScaled.x,
            y: this.state.pan.y + diffScaled.y
        };
    }

    resetPan() {
        this.state.pane = null;
        this.state.pan = {
            x: 0,
            y: 0
        };
    }

    render() {
        return (
            <Container>
                <Row>
                    <Col sm={12} md={{ size: 9, offset: 0 }}>
                        <div id="map-container" className="map-container"><div id='map' /></div>
                        <SaveInfo map={this.state.map} save={this.state.save} />
                        {this.renderSpinner()}
                        <Alert color="danger" isOpen={this.state.fileError} toggle={_ => {
                            this.setState({fileError: false})
                        }}>
                            {this.state.fileErrorMsg}
                        </Alert>
                        <Input type='file' name='file' id='file' onChange={this.handleFileSelected}/>
                    </Col>
                </Row>
            </Container>
        )
    }

    renderSpinner() {
        if (this.state.loading) {
            return (
                <Spinner className='mt-2' color="primary"/>
            )
        }
    }

    clickFileInput() {
        var fileInput = document.getElementById('file');
        if(fileInput)
             fileInput.click();
    }

    handleFileSelected(event) {
        let file = event.target.files[0];

        if (!file)
            return;

        let extension = file.name.split('.').pop();

        if (extension !== 'brs') {
            this.setState({
                fileError: true,
                fileErrorMsg: "File must be Brickadia save format (.brs)"
            });
            return;
        }

        this.loadFile(file);
    }

    loadFile(file) {
        this.setState({
            fileError: false,
            loading: true,
            map: removeFileExtension(file.name)
        }, () => {
            this.loadFileWASM(file);
        });
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
                    this.state.save.processBricks(this.state.showOutlines, this.state.fillBricks);
                    this.setState({loading: false});
                } catch (err) {
                    this.setState({
                        loading: false,
                        fileError: true,
                        fileErrorMsg: err
                    });
                }
                
                if (resetPan)
                    this.resetPan();
                this.canvas.needRedraw();
            }
        );
    }

}
