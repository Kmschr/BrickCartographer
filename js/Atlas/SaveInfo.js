import React, {Component} from "react";
import {Jumbotron, Container} from "reactstrap";
import {numberWithCommas} from "../util";

export default class SaveInfo extends Component {

    render() {
        if (this.props.save) {
            return (
                <Jumbotron id="saveInfo" fluid>
                    <Container fluid>
                        <h1 className="display-4">{this.props.map}</h1>
                        <hr className="my-2" />
                        {this.renderBrickCount()}
                        <p>{this.props.save.description()}</p>
                    </Container>
                </Jumbotron>
            )
        }
        return null;
    }

    renderBrickCount() {
        if (this.props.save) {
            return (
                <div>{numberWithCommas(this.getBrickCount()) + " bricks"}</div>
            )
        }
    }

    getBrickCount() {
        let bricks = 0;
        try {
            bricks = this.props.save.brickCount();
        } catch (err) {
            console.error(err);
        }
        return bricks;
    }

}