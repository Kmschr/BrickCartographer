import React, {Component} from 'react';
import {Col, Container, Row, Input, Spinner, Alert, Button} from 'reactstrap';
import {drawBricksContext2D} from "./RenderingContext2D";
import {RGBToHSL} from "./colors";
import 'leaflet/dist/leaflet.css';
import '../mapstyle.css';
import './L.CanvasLayer';
import 'brs-js';

const MAP_CENTER_DEFAULT = [0, 0];
const MAP_ZOOM_DEFAULT = 0;
const MAP_ZOOM_MIN = -5;

const wasm = import('../wasm');

export default class Atlas extends Component {

    constructor(props) {
        super(props);
        this.loadFile = this.loadFile.bind(this);
        this.loadFileWASM = this.loadFileWASM.bind(this);
        //this.readBuff = this.readBuff.bind(this);
        this.onDrawLayer = this.onDrawLayer.bind(this);
        this.state = {
            fileExtensionError: false,
            loading: false,
            bounds: null,
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

        // Add a 2D Rendering Context to the Map
        L.canvasLayer().delegate(this).addTo(this.map);
    }

    // Called upon any pan/zoom of map by canvas layer library
    onDrawLayer(info) {
        drawBricksContext2D(info, this.state.save, this.state.bounds, this.map);
    }

    render() {
        return (
            <Container>
                <Row>
                    <Col sm={12} md={{size: 9, offset: 0}}>
                        <div id='map'/>
                        {this.renderSpinner()}
                        <h2>{this.state.map}</h2>
                        <p>{this.state.description}</p>
                        <Alert color="danger" isOpen={this.state.fileExtensionError} toggle={_ => {
                            this.setState({fileExtensionError: false})}}>
                            File must be Brickadia save format (.brs)
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

    drawMethodGL(info) {
        const gl = info.canvas.getContext('webgl', {
            antialias: false,
            desynchronized: true
        });
        gl.clearColor(0.9, 0.9, 0.9, 1);
        gl.clear(gl.COLOR_BUFFER_BIT);
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
            map: file.name
        }, () => {
            this.loadFileWASM(file);
        });
    }

    loadFileWASM(file) {
        file.arrayBuffer()
            .then(buff => new Uint8Array(buff))
            .then(buff => wasm.then(brs => brs.load_file(buff)).catch(console.error))
            .then(brs => {
                let save = {
                    description: brs.description(),
                    brick_count: brs.brick_count(),
                    colors: brs.colors(),
                    brick_assets: brs.brick_assets(),
                    bricks: []
                };

                let rustBricks = brs.bricks();

                for (let i=0; i < rustBricks.length; i++) {
                    let brick = rustBricks[i];
                    save.bricks[i] = {
                        asset_name_index: brick.asset_name_index(),
                        size: brick.size(),
                        position: brick.position(),
                        direction: brick.direction(),
                        rotation: brick.rotation(),
                        visibility: brick.visibility(),
                        color: brick.color()
                    }
                }

                console.log(save);
                let bounds = this.getBounds(save);

                this.setState({
                    description: save.description,
                    save: save,
                    bounds: bounds
                }, () => {
                    this.setState({loading: false});
                    this.map.flyTo(L.latLng(0, 0), -1);
                });
            });
    }

    /*
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
            map: file.name
        }, () => {
            file.arrayBuffer().then(this.readBuff);
        });
    }


    readBuff(buff) {
        let save = BRS.read(new Buffer(buff));
        save.bricks.sort((a, b) => {
           return (a.position[2]+a.size[2]) -  (b.position[2]+b.size[2]);
        });
        save.colors = save.colors.map((rgb) => {
            return RGBToHSL(rgb[0], rgb[1], rgb[2]);
        });
        let bounds = this.getBounds(save);

        //console.log(save);
        console.log("done");
        this.setState({
            description: save.description,
            save: save,
            bounds: bounds
        }, () => {
            this.setState({loading: false});
            this.map.flyTo(L.latLng(0, 0), -1);
        });
    }
     */

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

}
