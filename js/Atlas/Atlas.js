import React, { Component } from 'react';
import { Col, Container, Row, Input, Spinner, Alert, Jumbotron } from 'reactstrap';
import { removeFileExtension } from "../util";
import 'leaflet/dist/leaflet.css';
import '../mapstyle.css';
import './L.CanvasLayer';
import 'brs-js';
import SaveInfo from "./SaveInfo";

const MAP_CENTER_DEFAULT = [0, 0];
const MAP_ZOOM_DEFAULT = 0;
const MAP_ZOOM_MIN = -5;

const wasm = import('../../pkg');

export default class Atlas extends Component {

    constructor(props) {
        super(props);
        this.loadFile = this.loadFile.bind(this);
        this.loadFileWASM = this.loadFileWASM.bind(this);
        this.onDrawLayer = this.onDrawLayer.bind(this);
        this.getNewPan = this.getNewPan.bind(this);
        this.state = {
            fileExtensionError: false,
            fileReadError: false,
            fileReadErrorMsg: null,
            loading: false,
            map: null,
            save: null,
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
            scrollWheelZoom: true
        });

        // Add a HTMLCanvas to the Map
        L.canvasLayer().delegate(this).addTo(this.map);
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
                        <div id='map' />
                        <SaveInfo map={this.state.map} save={this.state.save} />
                        {this.renderSpinner()}
                        <Alert color="danger" isOpen={this.state.fileExtensionError} toggle={_ => {
                            this.setState({ fileExtensionError: false })
                        }}>
                            File must be Brickadia save format (.brs)
                        </Alert>
                        <Alert color="danger" isOpen={this.state.fileReadError} toggle={_ => {
                            this.setState({ fileReadError: false })
                        }}>
                            {this.state.fileReadErrorMsg}
                        </Alert>
                        <p className="mt-2">
                            Load Brickadia Save:
                            <Input type='file' name='file' onChange={this.loadFile} />
                        </p>
                    </Col>
                </Row>
            </Container>
        )
    }

    renderSpinner() {
        if (this.state.loading) {
            return (
                <Spinner className='mt-2' color="primary" />
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
                try {
                    let center = save.process_bricks();
                    console.log(center);
                    this.setState({
                        save: save
                    }, () => {
                        this.setState({ loading: false });
                        this.map.setView(L.latLng(0, 0), -2);
                    });
                } catch (err) {
                    console.error(err);
                }
            });
    }

}
