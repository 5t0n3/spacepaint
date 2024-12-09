var map = {};
var mode = {"ctrl_clouds": null, "ctrl_heat": null, "ctrl_wind": null};
var mode_view = {"view_clouds": true, "view_heat": true, "view_wind": true}
var laser_width = 15;
let objects = [];

function toggleAboutModal() {
    document.getElementById("modal-backdrop").classList.toggle('hidden');
    document.getElementById("about-modal").classList.toggle('hidden');
}

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
      }else{
        myPolyline.remove(map)
        let coords=myPolyline.getLatLngs()
        for (coord of coords){
            console.log(coord)
        }
        //send coords of drawing
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

    //indexed "yx"
    let polygons=[]
    let Polygons=[]
    map.on('move', function() {
        for (P of Polygons) {
            P.remove(map);
        }
        Polygons=[]
        let array = [];
        let location=[];
        let Zoomlist=[20,16,9,6,4,1.5,1,0.5,0.2,0.1,0.05,0.03,0.02,0.01,0.005];
        let zoom=Zoomlist[map.getZoom()];
        //send page stuff
        for (let y = map.getCenter().lng-20*zoom; y < 20*zoom+map.getCenter().lng; y+=zoom) {
            let row = [];
            let xrow = [];
            for (let x = map.getCenter().lat-10*zoom; x < 12*zoom+map.getCenter().lat; x+=zoom) {
                row.push(Math.sin(x) * Math.cos(y)+0.5*Math.sin(100*x) * Math.cos(100*y));
                //get value (prolly outside of loop)
                xrow.push([x,y]);
            }
            array.push(row);
            location.push(xrow);
        }    
	    console.log(map.getCenter().lat,"lat")
        console.log(map.getCenter().lng,"lng")
        console.log(map.getZoom(),"zoom")
        
        
        for (let v = -1; v < 0; v += 1 / 5) {
            //console.log(Polygons)
            polygons = marchingSquares(array, v,location,zoom);
            for (p of polygons) {
                P=L.polygon(p, { color: "#0000ff", fillOpacity: 0.1, stroke: false });
                P.addTo(map);
                //console.log(P);
                Polygons.push(P);
            }
        }
        for (let v = 0; v < 1; v += 1 / 5) {
            //console.log(Polygons)
            polygons = marchingSquares(array, v,location,zoom);
            for (p of polygons) {
                P=L.polygon(p, { color: "#ff0000", fillOpacity: 0.1, stroke: false });
                P.addTo(map);
                //console.log(P);
                Polygons.push(P);
            }
        }        
    })


    // Makes a button for the UI
    function makeButton(html, tooltip, className, subActions, hookFn, hookFnArgs) {
        return L.Toolbar2.Action.extend({
            options: {
                toolbarIcon: {
                    html: html,
                    tooltip: tooltip,
                    className: className
                },
                subToolbar: new L.Toolbar2({ 
                    actions: subActions})
            },
            addHooks: function () {
                hookFn(...hookFnArgs);
            }
        });
    }
    
    // Toggles the view mode of a button and displays enable vs disabled colors
    function toggleMode_view(mode_var, mode_type, className) {
        mode_var[mode_type] = !mode_var[mode_type];
        button_html = document.getElementsByClassName(className)[0];
        if(mode_var[mode_type]){
            button_html.setAttribute("style", "background-color: #3737ff;");
        } else {
            button_html.setAttribute("style", "background-color: #919187;");
        }
    }
    // Toggles the edit mode of a button and displays add, remove, or disabled colors
    function toggleMode(mode_var, mode_type, className) {
        switch (mode_var[mode_type]){
            case true:
                mode_var[mode_type] = false;
                break;
            case false:
                mode_var[mode_type] = null;
                break;
            case null:
                mode_var[mode_type] = true;
                break;
        }
        if(mode_var[mode_type] !== null) {
            for(const item in mode_var) {
                if(item != mode_type) {
                    mode_var[item] = null;
                }
            }
        }
        for(const item in mode_var) {
            button_html = document.getElementsByClassName(item)[0];
            if(mode[item] === true){
                button_html.setAttribute("style", "background-color: #37ff37;");
            } else if (mode[item] === false) {
                button_html.setAttribute("style", "background-color: #ff3737;");
                
            } else {
                button_html.setAttribute("style", "background-color: #919187;");
            }
        }  
    }

    // Toggles display of the sub tool bar
    function toggleSubBar(subActionClass) {
        subBarHTML = document.getElementsByClassName(subActionClass)[0].parentElement.parentElement;
        subBarHTML.classList.toggle("hidden");
    }

    function slider_input(sliderClass) {
        laser_width = document.getElementsByClassName(sliderClass)[0].childNodes[0].value
    }

    // Button to go to about page
    var aboutPage = makeButton('&#9432;', 'About Page', 'about_button', [], toggleAboutModal, []);

    // Buttons to enable/diable view of clouds, heat, and wind
    var view_cloud = makeButton('&#9729;', 'View clouds', 'view_clouds', [], toggleMode_view, [mode_view, "view_clouds", 'view_clouds']);
    var view_heat = makeButton('&#127777;', 'View heat', 'view_heat', [], toggleMode_view, [mode_view, "view_heat", 'view_heat']);
    var view_wind = makeButton('&#x2248;', 'View wind', 'view_wind', [], toggleMode_view, [mode_view, "view_wind", 'view_wind']);
    // Button for dropdown for above buttons for viewing map
    var laser_view = makeButton('&#128065;', 'Control view', 'laser_view', [view_cloud, view_heat, view_wind], 
        toggleSubBar, ['view_clouds']);

    // Buttons to enable/diable editing of clouds, heat, and wind
    var control_cloud = makeButton('&#9729;', 'Edit clouds', 'ctrl_clouds', [], toggleMode, [mode, "ctrl_clouds", 'ctrl_clouds']);
    var control_heat = makeButton('&#127777;', 'Edit heat', 'ctrl_heat', [], toggleMode, [mode, "ctrl_heat", 'ctrl_heat']);
    var control_wind = makeButton('≈', 'Edit wind', 'ctrl_wind', [], toggleMode, [mode, "ctrl_wind", 'ctrl_wind']);
    // Button for dropdown for above buttons for editing map
    var control_laser = makeButton('&#128396;', 'Control laser', 'ctrl_laser', [control_cloud, control_heat, control_wind], 
        toggleSubBar, ['ctrl_clouds']);

    var width_slider = makeButton(`<input type="range" min="1" max="50" value="${laser_width}">`,
        'Control width slider', 'ctrl_slider', [], slider_input,  ['ctrl_slider']);
    var control_laser_width = makeButton('&#11044;', 'Control laser width', 'ctrl_laser_width', [width_slider], toggleSubBar, ['ctrl_slider']);

    
    // Create main tool bar
    new L.Toolbar2.Control({
        position: 'topleft',
        actions: [aboutPage, laser_view, control_laser, control_laser_width]
    }).addTo(map);

    // Initialize sub tool bars as hidden
    ctrlSubBarHTML = document.getElementsByClassName('ctrl_clouds')[0].parentElement.parentElement;
    ctrlSubBarHTML.classList.toggle("hidden");
    viewSubBarHTML = document.getElementsByClassName('view_clouds')[0].parentElement.parentElement;
    viewSubBarHTML.classList.toggle("hidden");
    sliderSubBarHTML = document.getElementsByClassName('ctrl_slider')[0].parentElement.parentElement;
    sliderSubBarHTML.classList.toggle("hidden");

    // Initialize display of view state
    for (const item of ["view_clouds", "view_heat", "view_wind"]) {
        if(mode_view[item]) {
            subBarButton_html = document.getElementsByClassName(item)[0];
            subBarButton_html.setAttribute("style", "background-color: #3737ff;");
        }
    }
    // Initialize display of enable state
    for (const item in mode) {
        subBarButton_html = document.getElementsByClassName(item)[0];
        if(mode[item] === true){
            subBarButton_html.setAttribute("style", "background-color: #37ff37;");
        } else if (mode[item] === false) {
            subBarButton_html.setAttribute("style", "background-color: #ff3737;");
            
        } else {
            subBarButton_html.setAttribute("style", "background-color: #919187;");
        }
    }



});

