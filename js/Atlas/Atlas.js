import React, {Component} from 'react';
import {Col, Container, Row, Input, Spinner, Alert, Button, CardLink} from 'reactstrap';
import {removeFileExtension} from "../util";
import 'leaflet/dist/leaflet.css';
import '../mapstyle.css';
import './L.CanvasLayer';
import 'brs-js';
import SaveInfo from "./SaveInfo";

const MAP_CENTER_DEFAULT = [0, 0];
const MAP_ZOOM_DEFAULT = 0;
const MAP_ZOOM_MIN = -5;

const FULLSCREEN_SYMBOL = "\u26F6";
const BRICK_OUTLINE_SYMBOL = "\u25A6";

const wasm = import('../../pkg');

export default class Atlas extends Component {

    constructor(props) {
        super(props);
        this.loadFile = this.loadFile.bind(this);
        this.loadFileWASM = this.loadFileWASM.bind(this);
        this.onDrawLayer = this.onDrawLayer.bind(this);
        this.getNewPan = this.getNewPan.bind(this);
        this.processSave = this.processSave.bind(this);
        this.getFullscreenButton = this.getFullscreenButton.bind(this);
        this.toggleFullscreen = this.toggleFullscreen.bind(this);
        this.getBrickOutlineButton = this.getBrickOutlineButton.bind(this);
        this.toggleBrickOutlines = this.toggleBrickOutlines.bind(this);
        this.state = {
            fileExtensionError: false,
            fileReadError: false,
            fileReadErrorMsg: null,
            loading: false,
            map: null,
            save: null,
            fullscreen: false,
            showOutlines: false,
            outlineButtonSymbol: BRICK_OUTLINE_SYMBOL,
            pan: {
                x: 0,
                y: 0
            }
        };
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

        // Add Fullscreen button
        L.Control.Fullscreen = L.Control.extend({ onAdd: this.getFullscreenButton });
        L.control.fullscreen = function(opts) {
            return new L.Control.Fullscreen(opts);
        }
        L.control.fullscreen({ position: 'topright' }).addTo(this.map);

        L.Control.BrickOutline = L.Control.extend({ onAdd: this.getBrickOutlineButton });
        L.control.brickOutline = function(opts) {
            return new L.Control.BrickOutline(opts);
        }
        L.control.brickOutline({ position: 'bottomleft' }).addTo(this.map);

        // Add a HTMLCanvas to the Map
        this.canvas = L.canvasLayer().delegate(this);
        this.canvas.addTo(this.map);
    }

    getFullscreenButton() {
        let button = L.DomUtil.create('input');
        L.DomUtil.addClass(button, "map-button");
        button.type = "button";
        button.title = "Fullscreen";
        button.value = FULLSCREEN_SYMBOL;

        button.onclick = this.toggleFullscreen;

        return button;
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

    getBrickOutlineButton() {
        let button = L.DomUtil.create('input');
        L.DomUtil.addClass(button, "map-button");
        button.type = "button";
        button.title = "Toggle Brick Outlines";
        button.value = BRICK_OUTLINE_SYMBOL;

        button.onclick = this.toggleBrickOutlines;

        return button;
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

    // Called upon any pan/zoom of map by canvas layer library
    onDrawLayer(info) {
        if (this.state.save) {
            // get current pan and current scale
            let pane = this.map._getMapPanePos();
            let scale = Math.pow(2, this.map.getZoom());

            let newPan = this.getNewPan(pane, scale);

            this.state.save.render(info.canvas.width, info.canvas.height, newPan.x, newPan.y, scale);

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

    render() {
        return (
            <Container>
                <Row>
                    <Col sm={12} md={{ size: 9, offset: 0 }}>
                        <div id="map-container" className="map-container"><div id='map' /></div>
                        <SaveInfo map={this.state.map} save={this.state.save} />
                        {this.renderSpinner()}
                        <Alert color="danger" isOpen={this.state.fileExtensionError} toggle={_ => {
                            this.setState({ fileExtensionError: false })
                        }}>
                            File must be Brickadia save format (.brs)
                        </Alert>
                        <Alert color="danger" isOpen={this.state.fileReadError} toggle={_ => {
                            this.setState({fileReadError: false})
                        }}>
                            {this.state.fileReadErrorMsg}
                        </Alert>
                        <p className="mt-2">
                            Load Brickadia Save:
                            <Input type='file' name='file' onChange={this.loadFile}/>
                        </p>
                        <br/>
                        <h2>Planned Features</h2>
                        <ul>
                            <li>Map rotation</li>
                            <li>Default save</li>
                            <li>Chunk rendering for improved performance</li>
                            <li>PNG exporting</li>
                            <li>Altitude cutoff for viewing inside structures</li>
                            <li>Color adjustment options</li>
                        </ul>
                        <h2>Known Issues</h2>
                        <ul>
                            <li>Certain saves will not load (e.g: Brickadia City) due to error w/ brs-rs</li>
                            <li>Zooming w/ mouse/shift-drag does not pan appropiately</li>
                        </ul>
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

    loadFile(event) {
        let file = event.target.files[0];
        let extension = file.name.split('.').pop();

        if (extension !== 'brs') {
            this.setState({
                fileExtensionError: true
            });
            return;
        }

        this.setState({
            fileExtensionError: false,
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
                wasm.then(rust => rust.load_file(buff)).catch((error) => {
                    this.setState({
                        fileReadError: true,
                        fileReadErrorMsg: error
                    });
                })
            )
            .then(save => {
                this.setState({
                    save: save,
                    fileReadError: false,
                }, () => {
                    this.processSave();
                    this.map.panTo([0,0]);
                });
            });
    }

    processSave() {
        this.setState({
            loading: true
        }, () => {
            try {
                this.state.save.process_bricks(this.state.showOutlines);
                this.setState({loading: false});
                this.canvas.needRedraw();
            } catch (err) {
                this.setState({
                    loading: false,
                    fileReadError: true,
                    fileReadErrorMsg: err
                });
            }
        });
    }

}
