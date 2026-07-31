// Build views: the 2x2 lumber skeleton with the plywood shown semi-transparent
// on top of it, so you can see which piece goes where.
//
//   dark brown = 2x2 lumber (38 x 38 mm dressed)
//   pale tan   = plywood (12 mm skin, 18 mm shelves)
//
// Doors and loose panels are pulled forward so they read as separate parts.

include <parts.scad>

module lumber(w, d, h, x, y, z) {
    color(C_LUMBER) box_at(w, d, h, x, y, z);
}

module ply(w, d, h, x, y, z) {
    color([C_PLY[0], C_PLY[1], C_PLY[2], PLY_ALPHA]) box_at(w, d, h, x, y, z);
}

// ------------------------------------------------------------- cabinet ------
POST_X = [SKIN_T, CABINET_W - SKIN_T - LUMBER];
POST_Y = [SKIN_T, CABINET_D - SKIN_T - LUMBER];

RAIL_SPAN_X = POST_X[1] - (POST_X[0] + LUMBER);
RAIL_SPAN_Y = POST_Y[1] - (POST_Y[0] + LUMBER);

FLOOR_Z = INTERIOR_Z0 - SHELF_T;

// One rectangle of 2x2 rails at height z, tying the four posts together.
module rail_ring(z) {
    for (y = POST_Y)
        lumber(RAIL_SPAN_X, LUMBER, LUMBER, POST_X[0] + LUMBER, y, z);
    for (x = POST_X)
        lumber(LUMBER, RAIL_SPAN_Y, LUMBER, x, POST_Y[0] + LUMBER, z);
}

module cabinet_frame(explode = true) {
    off = explode ? EXPLODE : 0;

    // four corner posts, floor to top
    for (x = POST_X, y = POST_Y)
        lumber(LUMBER, LUMBER, CABINET_H, x, y, 0);

    // rails: one ring under every horizontal panel
    rail_ring(FLOOR_Z - LUMBER);
    for (z = SHELF_Z) rail_ring(z - LUMBER);
    rail_ring(DIVIDER_Z - LUMBER);
    rail_ring(CABINET_H - SKIN_T - LUMBER);

    // plywood carcass
    ply(SKIN_T, CABINET_D, CABINET_H, 0, 0, 0);
    ply(SKIN_T, CABINET_D, CABINET_H, CABINET_W - SKIN_T, 0, 0);
    ply(CABINET_W - 2 * SKIN_T, SKIN_T, CABINET_H, SKIN_T, CABINET_D - SKIN_T, 0);
    ply(CABINET_W - 2 * SKIN_T, CABINET_D - SKIN_T, SKIN_T,
        SKIN_T, 0, CABINET_H - SKIN_T);

    // toe kick
    ply(CABINET_W - 2 * SKIN_T, SKIN_T, PLINTH_H, SKIN_T, PLINTH_INSET, 0);

    // horizontal plywood: floor, shelves, service divider
    ply(CABINET_W - 2 * SKIN_T, CABINET_D - SKIN_T, SHELF_T, SKIN_T, 0, FLOOR_Z);
    for (z = SHELF_Z)
        ply(CABINET_W - 2 * SKIN_T, CABINET_D - SKIN_T - SHELF_INSET, SHELF_T,
            SKIN_T, SHELF_INSET, z);
    ply(CABINET_W - 2 * SKIN_T, CABINET_D - SKIN_T, SHELF_T,
        SKIN_T, 0, DIVIDER_Z);

    // three doors and the service panel, pulled forward off the front
    for (i = [0 : DOOR_COUNT - 1])
        ply(CABINET_W, SKIN_T, DOOR_H, 0, -SKIN_T - off, DOOR_Z[i]);
    ply(CABINET_W, SKIN_T, CABINET_H - ELEC_Z0,
        0, -SKIN_T - off, ELEC_Z0);
}

// --------------------------------------------------------------- kiosk ------
KIOSK_LEG_X = [-BASE_W / 2 + LEG_INSET, BASE_W / 2 - LEG_INSET - LUMBER];
KIOSK_LEG_Y = [-BASE_D / 2 + LEG_INSET, BASE_D / 2 - LEG_INSET - LUMBER];

module kiosk_rail_ring(z) {
    span_x = KIOSK_LEG_X[1] - (KIOSK_LEG_X[0] + LUMBER);
    span_y = KIOSK_LEG_Y[1] - (KIOSK_LEG_Y[0] + LUMBER);
    for (y = KIOSK_LEG_Y)
        lumber(span_x, LUMBER, LUMBER, KIOSK_LEG_X[0] + LUMBER, y, z);
    for (x = KIOSK_LEG_X)
        lumber(LUMBER, span_y, LUMBER, x, KIOSK_LEG_Y[0] + LUMBER, z);
}

module kiosk_frame(explode = true) {
    off = explode ? EXPLODE : 0;
    lift = SU_D * tan(HEAD_TILT);

    // plywood base
    ply(BASE_W, BASE_D, SHELF_T, -BASE_W / 2, -BASE_D / 2, 0);

    // four 2x2 legs
    for (x = KIOSK_LEG_X, y = KIOSK_LEG_Y)
        lumber(LUMBER, LUMBER, STAND_H - SHELF_T, x, y, SHELF_T);

    // two rail rings brace the tall post
    kiosk_rail_ring(RAIL_Z);
    kiosk_rail_ring(STAND_H * 0.62);

    // plywood top plate and the tilt wedge
    ply(TOP_PLATE_W, TOP_PLATE_D, SHELF_T,
        -TOP_PLATE_W / 2, -TOP_PLATE_D / 2, STAND_H);
    color(C_LUMBER)
        translate([-SU_W / 2, -30, STAND_H + SHELF_T]) wedge(SU_W, SU_D, lift);

    // head as a plywood shell, lid pulled up so the inside is visible
    translate([-SU_W / 2, -30, STAND_H + SHELF_T])
        translate([0, SU_D, 0]) rotate([-HEAD_TILT, 0, 0]) translate([0, -SU_D, 0]) {
            ply(SKIN_T, SU_D, SU_H, 0, 0, 0);
            ply(SKIN_T, SU_D, SU_H, SU_W - SKIN_T, 0, 0);
            ply(SU_W - 2 * SKIN_T, SU_D, SKIN_T, SKIN_T, 0, 0);
            ply(SU_W - 2 * SKIN_T, SKIN_T, SU_H - 2 * SKIN_T,
                SKIN_T, SU_D - SKIN_T, SKIN_T);
            // face panel, pulled off the front
            ply(SU_W, SKIN_T, SU_H, 0, -SKIN_T - off, 0);
            // top panel, lifted clear
            ply(SU_W - 2 * SKIN_T, SU_D, SKIN_T,
                SKIN_T, 0, SU_H - SKIN_T + off * 0.5);
        }
}
