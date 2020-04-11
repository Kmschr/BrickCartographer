
// Render bricks using CanvasRenderingContext2D
// https://developer.mozilla.org/en-US/docs/Web/API/CanvasRenderingContext2D
export function drawBricksContext2D(info, save, bounds, map) {
    let ctx = info.canvas.getContext('2d');
    ctx.clearRect(0, 0, info.canvas.width, info.canvas.height);
    console.log(info.canvas);

    if (!save || !bounds)
        return;

    ctx.save();
    let zoom = Math.pow(2, map.getZoom());
    let panePos = map._getMapPanePos();

    // render middle of map to middle of canvas and apply offsets
    let translatePos = {
        x: info.canvas.width/2 - (bounds.x2 - bounds.x1)*zoom/(2),
        y: info.canvas.height/2 - (bounds.y2 - bounds.y1)*zoom/(2),
    };

    ctx.translate(translatePos.x, translatePos.y);
    ctx.scale(zoom, zoom);

    for (let i=0; i < save.bricks.length; i++) {
        let brick = save.bricks[i];

        // ignore invisible bricks
        if (!brick.visibility)
            continue;

        let name = save.brick_assets[brick.asset_name_index];
        if (name[0] !== 'P')
            continue;

        if (!Array.isArray(brick.color)) {
            if (brick.color === -1) {
                console.log("old color style");
                return;
            }
            let color = save.colors[brick.color];
            ctx.fillStyle = color;

            let x, y, width, height;

            if (brick.rotation === 0 || brick.rotation === 2) {
                x = brick.position[0] - brick.size[0] - bounds.x1;
                y = brick.position[1] - brick.size[1] - bounds.y1;
                width = brick.size[0]*2;
                height = brick.size[1]*2;
            } else if (brick.rotation === 1 || brick.rotation === 3) {
                x = brick.position[0] - brick.size[1] - bounds.x1;
                y = brick.position[1] - brick.size[0] - bounds.y1;
                width = brick.size[1]*2;
                height = brick.size[0]*2;
            }

            //let coord = map.layerPointToContainerPoint([x, y]);
            let coord = {
                x: x + panePos.x/zoom,
                y: y + panePos.y/zoom
            };

            if (name.includes("Wedge")) {
                let wedge = new Path2D();

                if (brick.rotation === 0) {
                    wedge.moveTo(coord.x, coord.y);
                    wedge.lineTo(coord.x + width, coord.y);
                    wedge.lineTo(coord.x, coord.y + height);
                    wedge.closePath();
                } else if (brick.rotation === 1) {
                    wedge.moveTo(coord.x, coord.y);
                    wedge.lineTo(coord.x + width, coord.y);
                    wedge.lineTo(coord.x + width, coord.y + height);
                    wedge.closePath();
                } else if (brick.rotation === 2) {
                    wedge.moveTo(coord.x, coord.y + height);
                    wedge.lineTo(coord.x + width, coord.y);
                    wedge.lineTo(coord.x + width, coord.y + height);
                    wedge.closePath();
                } else if (brick.rotation === 3) {
                    wedge.moveTo(coord.x, coord.y);
                    wedge.lineTo(coord.x, coord.y + height);
                    wedge.lineTo(coord.x + width, coord.y + height);
                    wedge.closePath();
                }

                ctx.fill(wedge);
                ctx.fillStyle = 'gray';
                ctx.stroke(wedge);
            } else {
                ctx.fillRect(coord.x, coord.y, width, height);
                ctx.fillStyle = 'gray';
                ctx.strokeRect(coord.x, coord.y, width, height);
            }
        }
    }

    ctx.restore();

    return translatePos;
}
