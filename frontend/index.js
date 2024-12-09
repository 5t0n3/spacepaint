var map = {};
var mode = {"ctrl_clouds": null, "ctrl_heat": null, "ctrl_wind": null};
var mode_view = {"view_clouds": true, "view_heat": true, "view_wind": true}
var laser_width = 15;
let objects = [];

function toggleAboutModal() {
    document.getElementById("modal-backdrop").classList.toggle('hidden');
    document.getElementById("about-modal").classList.toggle('hidden');
}

window.addEventListener('DOMContentLoaded', function () {
    map = L.map('map').setView([51.505, -0.09], 13);

    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: "&copy; <a href='http://www.openstreetmap.org/copyright'>OpenStreetMap</a>",
        drawControl: true
    }).addTo(map);

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
    // Toggles the view mode of a button and displays add, remove, or disabled colors
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
        button_html = document.getElementsByClassName(className)[0];
        if(mode_var[mode_type] === true){
            button_html.setAttribute("style", "background-color: #37ff37;");
        } else if (mode_var[mode_type] === false) {
            button_html.setAttribute("style", "background-color: #ff3737;");
            
        } else {
            button_html.setAttribute("style", "background-color: #919187;");
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
    var control_wind = makeButton('~', 'Edit wind', 'ctrl_wind', [], toggleMode, [mode, "ctrl_wind", 'ctrl_wind']);
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
    for (const item of ["ctrl_clouds", "ctrl_heat", "ctrl_wind"]) {
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

