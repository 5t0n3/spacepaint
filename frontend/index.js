var map = {};
var mode = 'none';
let objects = [];

function nonZero(inp) {
    if (Math.abs(inp) < 1e-4) {
        return inp + 1e-3;
    }
    return inp;
}

function marchingSquares(field, threshold) {
    let cells = [];
    for (row of field) {
        let r = [];
        for (c of row) {
            r.push(c > threshold);
        }
        cells.push(r);
    }

    let width = field[0].length;
    let height = field.length;

    let cases = [];

    for (let y = 0; y < height - 1; y++) {
        let row = [];
        for (let x = 0; x < width - 1; x++) {
            let c = cells[y + 1][x] + (cells[y + 1][x + 1] << 1) + (cells[y][x + 1] << 2) + (cells[y][x] << 3)
            row.push(c);
        }
        cases.push(row);
    }

    let polygons = [];

    for ([y, row] of cases.entries()) {
        for ([x, item] of row.entries()) {

            let tl = field[y][x];
            let tr = field[y][x + 1];
            let bl = field[y + 1][x];
            let br = field[y + 1][x + 1];

            let leftLerp = (threshold - tl) / nonZero(bl - tl);
            let bottomLerp = (threshold - bl) / nonZero(br - bl);
            let rightLerp = (threshold - tr) / nonZero(br - tr);
            let topLerp = (threshold - tl) / nonZero(tr - tl);

            let leftPoint = [0, leftLerp];
            let bottomPoint = [bottomLerp, 1];
            let rightPoint = [1, rightLerp];
            let topPoint = [topLerp, 0];

            let topLeft = [0, 0];
            let bottomLeft = [0, 1];
            let bottomRight = [1, 1];
            let topRight = [1, 0];

            let polys = [
                [],
                [[leftPoint, bottomPoint, bottomLeft]],
                [[bottomPoint, rightPoint, bottomRight]],
                [[leftPoint, rightPoint, bottomRight, bottomLeft]],

                [[topPoint, topRight, rightPoint]],
                [[leftPoint, topPoint, topRight, rightPoint, bottomPoint, bottomLeft]],
                [[topPoint, topRight, bottomRight, bottomPoint]],
                [[leftPoint, topPoint, topRight, bottomRight, bottomLeft]],

                [[topLeft, topPoint, leftPoint]],
                [[topLeft, topPoint, bottomPoint, bottomLeft]],
                [[topLeft, topPoint, rightPoint, bottomRight, bottomPoint, leftPoint]],
                [[topLeft, topPoint, rightPoint, bottomRight, bottomLeft]],
                
                [[topLeft, topRight, rightPoint, leftPoint]],
                [[topLeft, topRight, rightPoint, bottomPoint, bottomLeft]],
                [[topLeft, topRight, bottomRight, bottomPoint, leftPoint]],
                [[topLeft, topRight, bottomRight, bottomLeft]]
            ];

            let new_polygons = polys[item];

            for (p of new_polygons) {
                for (point of p) {
                    point[0] += x;
                    point[1] += y;
                }
            }

            for (p of new_polygons) {
                polygons.push(p);
            }
        }
    }

    return polygons;
}

var bar;

window.addEventListener('DOMContentLoaded', function () {
    map = L.map('map').setView([10, 10], 5);

    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: "&copy; <a href='http://www.openstreetmap.org/copyright'>OpenStreetMap</a>",
        drawControl: true
    }).addTo(map);



    let array = [];

    for (let y = 0; y < 20; y++) {
        let row = [];
        for (let x = 0; x < 20; x++) {
            row.push(Math.sin(x) * Math.cos(y));
        }
        array.push(row);
    }

    for (let v = -1; v < 1; v += 2 / 7) {
        let polygons = marchingSquares(array, v);

        for (p of polygons) {
            L.polygon(p, { color: "#ff0000", fillOpacity: 0.1, stroke: false }).addTo(map);
        }
    }

    var action1 = L.Toolbar2.Action.extend({
        options: {
            toolbarIcon: {
                html: '<div id="AAAA">&#9873;</div>',
                tooltip: 'your mother.'
            }
        },
        addHooks: function () {
            mode = 'heat';
            document.getElementById("AAAA").innerHTML = "blah";
        }
    });

    bar = new L.Toolbar2.Control({
        position: 'topright',
        actions: [action1]
    }).addTo(map);

    console.log(bar.options.actions);
});

