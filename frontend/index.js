import init, { Rect, LatLong, Pixel, update_viewport, do_changes, rect, latlong } from "./png-decoder/pkg/png_decoder.js";

var map = {};
var mode = 'none';
let objects = [];

function nonZero(inp) {
    if (Math.abs(inp) < 1e-4) {
        return inp + 1e-3;
    }
    return inp;
}

function marchingSquares(field, threshold,location,zoom) {
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

            let leftPoint = [0, zoom*leftLerp];
            let bottomPoint = [zoom*bottomLerp, zoom*1];
            let rightPoint = [zoom*1, zoom*rightLerp];
            let topPoint = [zoom*topLerp, 0];

            let topLeft = [0, 0];
            let bottomLeft = [0, zoom*1];
            let bottomRight = [zoom*1, zoom*1];
            let topRight = [zoom*1, 0];

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
                    point[0] += location[y][x][0];
                    point[1] += location[y][x][1];
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

//indexed "yx"
let polygons=[]
let Polygons=[]

function update_map(data, width, area) {
    for (P of Polygons) {
        P.remove(map);
    }
    Polygons=[]
    let array = [];
    let location=[];
    let Zoomlist=[20,16,9,6,4,1.5,1,0.5,0.2,0.1,0.05,0.03,0.02,0.01,0.005];
    let zoom=Zoomlist[map.getZoom()];
    let height_px = data.length;
    //send page stuff
    let y_idx = 0;
    for (let y = area.bottom_right.lat; y < area.top_left.lat; y += (area.top_left.lat - area.bottom_right.lat) / height_px) {
        let row = [];
        let xrow = [];
        let x_idx = 0;
        for (let x = area.top_left.long; x < area.bottom_right.long; x += (area.top_left.long - area.bottom_right.long) / width) {
            row.push(data[y_idx][x_idx++]);
            //get value (prolly outside of loop)
            xrow.push([x,y]);
        }
        y_idx++;
        array.push(row);
        location.push(xrow);
    }
    console.log(map.getCenter().lat,"lat")
    console.log(map.getCenter().lng,"lng")
    console.log(map.getZoom(),"zoom")
    
    for (let v = 0; v < 255; v += 255 / 10) {
        //console.log(Polygons)
        polygons = marchingSquares(array, v,location,zoom);
        for (p of polygons) {
            P = L.polygon(p, { color: "#aa00ff", fillOpacity: 0.1, stroke: false });
            P.addTo(map);
            //console.log(P);
            Polygons.push(P);
        }
    }
}

window.addEventListener('DOMContentLoaded', function () {
    map = L.map('map').setView([10, 10], 5);

    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: "&copy; <a href='http://www.openstreetmap.org/copyright'>OpenStreetMap</a>",
        drawControl: true
    }).addTo(map);


    let paintMode = false;
    var myPolyline;
    
    map.on('click', function() {
        paintMode = !paintMode;
          if (paintMode) {
              myPolyline = L.polyline([]).addTo(map);
          } else {
              myPolyline.remove(map)
              let coords = myPolyline.getLatLngs()
              let points = [];
              for (coord of coords){
                  points.push(latlong(coord.lat, coord.lng));
              }

              do_changes(points, brush_size, mode);
        }
    })
    
    map.on('mousemove', function(e) {
        if (paintMode) {
          myPolyline.addLatLng(e.latlng);
          console.log(myPolyline.setStyle({
            color: "#fff",
            weight: 60,
            opacity: 0.8
            }),"styles");
      //console.log(myPolyline.getLatLngs())
      }
    })

    map.on('move', function() {
        let bounds = map.getBounds();
        console.log(bounds);
        update_viewport(rect(bounds.getNorth(), bounds.getWest(), bounds.getSouth(), bounds.getEast()));
    });


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

    init();
});

