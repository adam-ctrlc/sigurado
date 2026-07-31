// Reusable geometry helpers. Every module takes the minimum corner (x, y, z)
// so the layout maths in the model files stays readable.

include <config.scad>

module box_at(w, d, h, x = 0, y = 0, z = 0) {
    translate([x, y, z]) cube([w, d, h]);
}

// Cylinder whose axis runs along +Z, starting at z, centred on (x, y).
module cyl_z(r, len, x, y, z) {
    translate([x, y, z]) cylinder(h = len, r = r);
}

// Cylinder whose axis runs along +Y, starting at y, centred on (x, z).
module cyl_y(r, len, x, y, z) {
    translate([x, y, z]) rotate([-90, 0, 0]) cylinder(h = len, r = r);
}

// Cylinder whose axis runs along +X, starting at x, centred on (y, z).
module cyl_x(r, len, x, y, z) {
    translate([x, y, z]) rotate([0, 90, 0]) cylinder(h = len, r = r);
}

// Open-topped tray / bin, used for the material zones.
module tray(w, d, h, x, y, z, wall = 2.5) {
    difference() {
        box_at(w, d, h, x, y, z);
        box_at(w - 2 * wall, d - 2 * wall, h, x + wall, y + wall, z + wall);
    }
}

// Extruded text on a vertical plane facing the front (-Y).
module front_label(txt, x, z, size = LABEL_SIZE) {
    translate([x, 0, z])
        rotate([90, 0, 0])
            linear_extrude(height = LABEL_DEPTH)
                text(txt, size = size, font = LABEL_FONT, halign = "center");
}

// Right-angle wedge: height h at the front (y = 0), tapering to 0 at the back.
// Used as the tilt bracket under the scanner head.
module wedge(w, d, h) {
    translate([w, 0, 0])
        rotate([0, -90, 0])
            linear_extrude(height = w)
                polygon([[0, 0], [h, 0], [0, d]]);
}
