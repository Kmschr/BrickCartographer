import React, { Component } from "react";
import { Container } from "reactstrap";

import "./header-footer.css";

const UNICODE_LINK_SYMBOL = "\uD83D\uDD17";
const UNICODE_WARNING_SIGN = "\u26A0";

export default class Footer extends Component {

    render() {
        const linkStatusSymbol = this.getSymbolFromConnectionStatus();
        return (
            <div className="full-width footer">
                <div className="vertical-center tco-text">
                    <Container>
                        <div className="centered">
                            {linkStatusSymbol}
                            Made by <a href="https://github.com/Kmschr">Kmschr</a> (Smallguy)
                        </div>
                    </Container>
                </div>
            </div>
        );
    }

    getSymbolFromConnectionStatus() {
        return UNICODE_LINK_SYMBOL;
    }

}
