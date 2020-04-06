import React, {Component} from 'react';
import {Container} from "reactstrap";

import './header-footer.css';
import HeaderLogo from './resources/brickadiaLogo.svg';

const BRICKADIA_URL = "https://brickadia.com/";

export default class Header extends Component {

    render() {
        return (
            <div className="full-width header">
                <div className="vertical-center">
                    <Container>
                        <div className="vertical-center">
                            <a href={BRICKADIA_URL} target="_blank">
                                <img className="tco-logo" src={HeaderLogo} alt="Brickadia Logo"/>
                            </a>
                            <h1 className="tco-text-upper">Brick Cartographer</h1>
                        </div>
                    </Container>
                </div>
            </div>
        );
    }
}
