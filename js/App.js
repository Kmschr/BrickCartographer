import React, { Component } from "react";
import {Container, Row, Col} from "reactstrap";

import "bootstrap/dist/css/bootstrap.css";
import "./Atlas/EasyButton/easy-button.css";

import Header from "./Margins/Header";
import Atlas from "./Atlas/Atlas";
import Footer from "./Margins/Footer";

export default class App extends Component {

    render() {
        return (
            <div>
                <Header/>
                <Atlas/>
                <Container>
                    <Row>
                        <Col sm={12} md={{ size: 9, offset: 0 }}>
                            <p>Message Smallguy#7841 on discord if you have a brs that you want to be featured as a default save</p>
                            <p>Use <a href="https://www.reheatedcake.io/bls2brs/">bls2brs</a> if you want to try and load a save from Blockland, results may vary</p>
                            <h2>Planned Features</h2>
                            <ul>
                                <li>Map rotation</li>
                                <li>Default saves</li>
                                <li>Chunk rendering for improved performance</li>
                                <li>Image exporting</li>
                                <li>Altitude cutoff for viewing inside structures</li>
                                <li>Color adjustment options</li>
                            </ul>
                            <h2>Known Issues</h2>
                            <ul>
                                <li>Certain saves will not load (e.g: Brickadia City) due to error w/ brs-rs</li>
                                <li>Zooming w/ mouse/shift-drag does not pan appropiately</li>
                                <li>Switching between fullscreen and standard views will not adjust view to center</li>
                                <li>Many unsupported bricks</li>
                                <li>Incorrect outlines on certain bricks</li>
                            </ul>
                            <h2>Release Notes</h2>
                            <h3>v0.1</h3>
                            <ul>
                                <li>saves are now loaded from rust version of brs</li>
                                <li>bricks are now sorted for rendering by top vertex height</li>
                                <li>rendering is now done from a wasm WebGL context</li>
                                <li>many procedural bricks are now supported</li>
                                <ul>
                                    <li>all rectangular bricks</li>
                                    <li>all wedge bricks</li>
                                    <li>basic ramp bricks</li>
                                </ul>
                                <li>a few non procedural bricks are now supported</li>
                                <li>issue with panning speed and zoom center is fixed</li>
                                <li>canvas resizing now works properly</li>
                                <li>leaflet control for fullscreen mode added</li>
                                <li>leaflet control for brick outlines added</li>
                            </ul>
                        </Col>
                    </Row>
                </Container>
                <Footer/>
            </div>
        );
    }

}

