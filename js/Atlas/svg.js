import React from "react";
import {SVGOverlay} from "react-leaflet";

// Using SVG format was first approach to render the bricks in the leaflet map
// by using an SVGOverlay
// It was found to be low performance solution and so stays here unused

/*
renderSVGLayer() {
    return (
        <SVGOverlay bounds={this.state.svgBounds}>
            {this.getSaveSVG(this.state.save, this.state.bounds)}
        </SVGOverlay>
    )
}
*/

export function getSaveSVG(save, bounds) {
    if(!save || !bounds)
        return null;
    let width = bounds.x2 - bounds.x1;
    let height = bounds.y2 - bounds.y1;
    let viewBox = bounds.x1 + ' ' + bounds.y1 + ' ' + width + ' ' + height;
    return (
        <svg xlmns="http://www.w3.org/2000/svg"
             viewBox={viewBox}>
            {getBricks(save)}
        </svg>
    )
}

function stressRender() {
    let rects = [];
    for (let x=0; x < 1000; x++) {
        for (let y=0; y < 100; y++) {
            let color = 'rgb(' + (x%255) + ',' + (y%255) + ',' + (100) + ')';
            let brick = {
                position: [x*10, y*10, 0],
                size: [10, 10, 1]
            };
            rects.push(getBrickSVG(brick, color))
        }
    }
    return rects;
}

function getBricks(save) {
    let rects = [];
    for (let i=0; i < save.bricks.length; i++) {
        let brick = save.bricks[i];

        // ignore invisible bricks
        if (!brick.visibility)
            continue;

        let name = save.brick_assets[brick.asset_name_index];
        if (name[0] !== 'P')
            continue;

        if (!Array.isArray(brick.color)) {
            let rgb = save.colors[brick.color];
            let color = 'rgb(' + rgb[0] + ',' + rgb[1] + ',' + rgb[2] + ')';
            rects.push(getBrickSVG(brick, color));
        }
    }
    return rects;
}


function getBrickSVG(brick, color) {
    let x = brick.position[0] - brick.size[0];
    let y = brick.position[1] - brick.size[1];
    let width = brick.size[0]*2;
    let height = brick.size[1]*2;
    let key = brick.position[0] + ',' + brick.position[1] + ',' + brick.position[2];
    return (
        <rect key={key} x={x} y={y} width={width} height={height} fill={color}/>
    )
}