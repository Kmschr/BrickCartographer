import React, {Component} from 'react';
import {Col, Container, Row, Input, Spinner, Alert, Jumbotron} from 'reactstrap';
import {numberWithCommas, removeFileExtension} from "../util";
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
        //this.readBuff = this.readBuff.bind(this);
        this.onDrawLayer = this.onDrawLayer.bind(this);
        this.state = {
            fileExtensionError: false,
            fileReadError: false,
            fileReadErrorMsg: null,
            loading: false,
            map: null,
            save: null
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
            scrollWheelZoom: false
        });

        // Add a HTMLCanvas to the Map
        L.canvasLayer().delegate(this).addTo(this.map);
    }

    // Called upon any pan/zoom of map by canvas layer library
    onDrawLayer(info) {
        if (this.state.save) {
            let panePos = this.map._getMapPanePos();
            this.state.save.render(this.map.getZoom(), panePos.x, panePos.y);
        }
    }

    render() {
        return (
            <Container>
                <Row>
                    <Col sm={12} md={{size: 9, offset: 0}}>
                        <div id='map'/>
                        <SaveInfo map={this.state.map} save={this.state.save}/>
                        {this.renderSpinner()}
                        <Alert color="danger" isOpen={this.state.fileExtensionError} toggle={_ => {
                            this.setState({fileExtensionError: false})}}>
                            File must be Brickadia save format (.brs)
                        </Alert>
                        <Alert color="danger" isOpen={this.state.fileReadError} toggle={_ => {
                            this.setState({fileReadError: false})}}>
                            {this.state.fileReadErrorMsg}
                        </Alert>
                        <p className="mt-2">
                            Load Brickadia Save:
                            <Input type='file' name='file' onChange={this.loadFile}/>
                        </p>
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
                }, () => {
                    this.setState({loading: false});
                    //this.map.flyTo(L.latLng(0, 0), -1);
                });
            });
    }

    /* REWRITE IN RUST
    getBounds(save) {
        let bounds = {
            x1: null,
            y1: null,
            x2: null,
            y2: null
        };
        for (let i=0; i < save.bricks.length; i++) {
            let brick = save.bricks[i];

            // ignore invisible bricks
            if (!brick.visibility)
                continue;

            let name = save.brick_assets[brick.asset_name_index];
            if (name[0] !== 'P')
                continue;

            // calculate bounds of procedural brick
            let x1 = brick.position[0] - brick.size[0];
            let x2 = brick.position[0] + brick.size[0];
            let y1 = brick.position[1] - brick.size[1];
            let y2 = brick.position[1] + brick.size[1];

            // no bounds yet so start with first brick bounds
            if (!bounds.x1) {
                bounds = {
                    x1: x1,
                    y1: y1,
                    x2: x2,
                    y2: y2
                }
            } else {
                if (x1 < bounds.x1)
                    bounds.x1 = x1;
                if (y1 < bounds.y1)
                    bounds.y1 = y1;
                if (x2 > bounds.x2)
                    bounds.x2 = x2;
                if (y2 > bounds.y2)
                    bounds.y2 = y2;
            }
        }
        return bounds;
    }
     */

}
