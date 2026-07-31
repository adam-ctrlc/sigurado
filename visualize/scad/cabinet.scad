// Model A: the Sigurado storage cabinet.
//
// Bottom to top: plinth -> 3 material compartments behind lockable double
// doors -> a CONCEALED SERVICE BAY behind a screwed panel. Every electronic
// part lives in that bay, so the end user only ever sees the material shelves.

include <parts.scad>

module cabinet_carcass() {
    color(C_PLINTH)
        box_at(CABINET_W, CABINET_D - PLINTH_INSET, PLINTH_H, 0, PLINTH_INSET, 0);

    color(C_CARCASS) {
        box_at(CABINET_W, CABINET_D, PANEL_T, 0, 0, PLINTH_H);
        box_at(CABINET_W, CABINET_D, PANEL_T, 0, 0, CABINET_H - PANEL_T);
        box_at(PANEL_T, CABINET_D, INTERIOR_Z1 - INTERIOR_Z0, 0, 0, INTERIOR_Z0);
        box_at(PANEL_T, CABINET_D, INTERIOR_Z1 - INTERIOR_Z0,
               CABINET_W - PANEL_T, 0, INTERIOR_Z0);
    }

    color(C_BACK)
        box_at(INTERIOR_W, BACK_T, INTERIOR_Z1 - INTERIOR_Z0,
               INTERIOR_X0, CABINET_D - BACK_T, INTERIOR_Z0);

    // sealed divider between the user storage and the electronics bay
    color(C_SERVICE)
        box_at(INTERIOR_W, INTERIOR_D, PANEL_T, INTERIOR_X0, 0, DIVIDER_Z);

    color(C_SHELF)
        for (z = SHELF_Z)
            box_at(INTERIOR_W, INTERIOR_D - SHELF_INSET, PANEL_T,
                   INTERIOR_X0, SHELF_INSET, z);
}

module cabinet_service_bay() {
    plate_z = ELEC_Z0;
    z = ELEC_Z0 + 4;
    y = 34;
    x_psu      = INTERIOR_X0 + 24;
    x_relay    = x_psu + PSU[0] + 20;
    x_mcu      = x_relay + RELAY[0] + 20;
    x_terminal = x_mcu + MCU[0] + 20;
    board_z    = z + 8;

    color(C_PLATE)
        box_at(INTERIOR_W - 20, INTERIOR_D - 40, 4, INTERIOR_X0 + 10, 20, plate_z);

    color(C_PSU)      box_at(PSU[0], PSU[1], PSU[2], x_psu, y, z);
    color(C_RELAY)    box_at(RELAY[0], RELAY[1], RELAY[2], x_relay, y, z);
    color(C_MCU)      box_at(MCU[0], MCU[1], MCU[2], x_mcu, y, board_z);
    color(C_TERMINAL) box_at(TERMINAL[0], TERMINAL[1], TERMINAL[2], x_terminal, y, z);

    color(C_PLATE)
        for (dx = [5, MCU[0] - 5], dy = [4, MCU[1] - 4])
            cyl_z(3, 8, x_mcu + dx, y + dy, z);

    color(C_WIRE) {
        cyl_x(9, INTERIOR_W - 40, INTERIOR_X0 + 20, INTERIOR_D - 40, z + 26);
        cyl_z(5, 46, CABINET_W / 2, INTERIOR_D - 40, DIVIDER_Z - 44);
    }
}

module cabinet_service_panel() {
    bay_h = CABINET_H - ELEC_Z0;

    color(C_SERVICE)
        box_at(CABINET_W, SERVICE_PANEL_T, bay_h, 0, -SERVICE_PANEL_T, ELEC_Z0);

    color(C_HANDLE)
        for (x = [26, CABINET_W - 26], z = [ELEC_Z0 + 26, CABINET_H - 30])
            cyl_y(4, 6, x, -SERVICE_PANEL_T - 2, z);

    // sits low on the panel so the scanner mounted on top of it stays clear
    if (SHOW_LABELS)
        color(C_LABEL)
            front_label("ELECTRONICS - NO USER ACCESS", CABINET_W / 2,
                        ELEC_Z0 + 11, 11);
}

LOCK_X = CABINET_W - PANEL_T - LOCK_W - 20;
LOCK_Y = 6;
LOCK_Z = DIVIDER_Z - LOCK_H;
BOLT_X = LOCK_X + LOCK_W / 2;
BOLT_Y = LOCK_Y + LOCK_D / 2;

// A single 12V solenoid, fail-secure: de-energised means the bolt is out.
// It hangs under the service divider at the door's free edge and throws its
// bolt down through the latch tab on the inside of the door.
module cabinet_lock() {
    color(C_LOCK) box_at(LOCK_W, LOCK_D, LOCK_H, LOCK_X, LOCK_Y, LOCK_Z);
    color(C_BOLT)
        cyl_z(LOCK_BOLT_DIA / 2, LOCK_BOLT_LEN,
              BOLT_X, BOLT_Y, LOCK_Z - LOCK_BOLT_LEN);
}

// Limit switch on the inside of the right panel, opposite the hinge, where a
// door is most likely to sit ajar. Its lever faces the door opening, so the
// closing door presses it through the striker pad on the door itself.
module cabinet_limit_switch() {
    x = CABINET_W - PANEL_T - SWITCH_W;
    body_y = STRIKER_D + LEVER_D;

    color(C_SWITCH) box_at(SWITCH_W, SWITCH_D, SWITCH_H, x, body_y, SENSE_Z);

    // The lever reaches back toward the door, where the striker meets it.
    color(C_SWITCH)
        box_at(LEVER_W, LEVER_D, LEVER_T,
               x + (SWITCH_W - LEVER_W) / 2, STRIKER_D,
               SENSE_Z + SWITCH_H - LEVER_T * 2);

    color(C_WIRE)
        cyl_z(2.5, DIVIDER_Z - SENSE_Z - SWITCH_H,
              x + SWITCH_W / 2, body_y + SWITCH_D / 2, SENSE_Z + SWITCH_H);
}

// Scan-at-the-box unit, mounted on the FRONT of the service panel, centred.
// It sits directly on the face of the concealed bay, so the sensor and relay
// wiring passes straight through into the electronics behind it.
module cabinet_box_node() {
    bay_h = CABINET_H - ELEC_Z0;
    x0 = CABINET_W / 2 - NODE_W / 2;
    y0 = -SERVICE_PANEL_T - NODE_D;
    z0 = ELEC_Z0 + (bay_h - NODE_H) / 2;
    cx = CABINET_W / 2;

    color(C_ENCL)   box_at(NODE_W, NODE_D, NODE_H, x0, y0, z0);
    color(C_SENSOR) cyl_y(13, 5, cx, y0 - 2, z0 + NODE_H * 0.32);
    color(C_LCD)    box_at(58, 3, 20, cx - 29, y0 - 1, z0 + NODE_H * 0.62);
    color(C_LED)    cyl_y(3, 4, cx + 38, y0 - 1, z0 + NODE_H * 0.32);
}

// A single slot, so the materials share it: parts bins across the front, a
// couple of wire spools and a kit box behind them.
module cabinet_materials() {
    gap = 18;
    z = INTERIOR_Z0;
    x0 = INTERIOR_X0 + gap;
    usable_w = INTERIOR_W - 2 * gap;
    front_d = (INTERIOR_D - gap) * 0.42;
    back_y = SHELF_INSET + gap + front_d + gap;

    bw = (usable_w - 2 * gap) / 3;
    color(C_MAT_COMPONENTS)
        for (col = [0 : 2])
            tray(bw, front_d, 110, x0 + col * (bw + gap), SHELF_INSET + gap, z);

    color(C_MAT_BULK)
        for (col = [0 : 1])
            cyl_y(42, 50, x0 + 55 + col * 100, back_y, z + 42);

    color(C_MAT_INSTRUMENTS)
        box_at(usable_w * 0.44, INTERIOR_D - back_y - 10, 140,
               x0 + usable_w * 0.56, back_y, z);
}

module cabinet_labels() {
    if (SHOW_LABELS)
        color(C_LABEL)
            front_label(SLOT_LABEL, CABINET_W / 2, INTERIOR_Z0 + 14);
}

// The door: hinged on the left, handle and latch tab on the right. The tab is
// ramped at its far end, so swinging the door shut cams the spring bolt up and
// lets it drop into the hole. That means closing never needs power, and the
// striker pad beside it presses the limit switch so the server knows the door is
// really home.
module cabinet_door(i) {
    z0 = DOOR_Z[i];
    handle_len = min(HANDLE_LEN, DOOR_H * 0.28);
    handle_x = CABINET_W - 42;
    handle_z = z0 + DOOR_H / 2 - handle_len / 2;
    tab_z = LOCK_Z - LOCK_BOLT_LEN;

    color(C_DOOR) box_at(CABINET_W, DOOR_T, DOOR_H, 0, -DOOR_T, z0);

    color(C_BOLT) {
        difference() {
            box_at(LOCK_W, TAB_FLAT_D, TAB_T, LOCK_X, 0, tab_z);
            cyl_z(LOCK_BOLT_DIA / 2 + 1, TAB_T + 4, BOLT_X, BOLT_Y, tab_z - 2);
        }
        translate([LOCK_X, TAB_FLAT_D, tab_z])
            wedge(LOCK_W, TAB_RAMP_D, TAB_T);
    }

    // Flat pad rather than the bare door edge, so the lever is pressed square on
    // and does not wear a groove in the door.
    color(C_STRIKER)
        box_at(STRIKER_W, STRIKER_D, STRIKER_H,
               CABINET_W - PANEL_T - STRIKER_W, 0, SENSE_Z);

    color(C_HANDLE) {
        cyl_z(HANDLE_DIA / 2, handle_len, handle_x, -DOOR_T - 24, handle_z);
        for (dz = [0, handle_len])
            cyl_y(6, 24, handle_x, -DOOR_T - 24, handle_z + dz);
    }
}

module cabinet(open_doors = true, service_open = false) {
    angle = open_doors ? DOOR_OPEN_DEG : 0;

    cabinet_carcass();
    cabinet_service_bay();
    if (!service_open) cabinet_service_panel();
    cabinet_lock();
    cabinet_limit_switch();
    cabinet_box_node();
    cabinet_materials();
    cabinet_labels();

    for (i = [0 : DOOR_COUNT - 1])
        rotate([0, 0, -angle]) cabinet_door(i);
}
