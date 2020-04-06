import React, { Component } from "react";

import "bootstrap/dist/css/bootstrap.css";

import Header from "./Margins/Header";
import Atlas from "./Atlas/Atlas";
import Footer from "./Margins/Footer";

export default class App extends Component {

    render() {
        return (
            <div>
                <Header/>
                <Atlas/>
                <Footer/>
            </div>
        );
    }

}

