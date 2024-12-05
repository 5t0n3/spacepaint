var map = {};
var mode = 'none';
let objects = [];

window.addEventListener('DOMContentLoaded', function () {
    map = L.map('map').setView([51.505, -0.09], 13);

    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: "&copy; <a href='http://www.openstreetmap.org/copyright'>OpenStreetMap</a>",
        drawControl: true
    }).addTo(map);

    var action1 = L.Toolbar2.Action.extend({
        options: {
            toolbarIcon: {
                html: '&#9873;',
                tooltip: 'your mother.'
            }
        },
        addHooks: function () {
            mode = 'heat';
        }
    });

    new L.Toolbar2.Control({
        position: 'topleft',
        actions: [action1]
    }).addTo(map);

    for (var x = 0; x < 25; x += 0.5) {
        for (var y = 0; y < 25; y += 0.5) {
            var line = L.polyline([
                [x, y],
                [x + 0.1, y + 0.1]
            ], { fillColor: '#ff0000' }).addTo(map);
        }
    }
});

