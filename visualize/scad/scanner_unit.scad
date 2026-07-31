// Model B: the door-side fingerprint scanner, on a four-leg floor stand.
//
// The head holds the fingerprint sensor, a 16x2 I2C LCD, the enroll button and
// a status LED. Everything else (ESP32-S3, buck converter, terminal block) is
// sealed inside, hidden from the end user. The head tilts back so the face
// points up at someone standing in front of it, like an ATM.
//
// The head is modelled with its front face on the y = 0 plane, facing -Y.

include <parts.scad>

FP_Z  = SU_H * FP_Z_FRAC;
LCD_Z = SU_H * LCD_Z_FRAC;
BTN_Z = SU_H * BTN_Z_FRAC;
THRU  = SU_WALL + 14;
CX    = SU_W / 2;

// Interior depth budget, front to back:
//   0..3      front wall
//   3..29     LCD module / sensor barrel, panel mounted
//   39..67    ESP32 and buck on standoffs off the back plate
//   67..70    back plate
BAY_Y   = SU_D - SU_BACKPLATE_T - STANDOFF_H - MOD_ESP[1];
BACK_IN = SU_D - SU_BACKPLATE_T;

// Box with the edges parallel to Y rounded off.
module rounded_box_y(w, d, h, r) {
    hull()
        for (x = [r, w - r], z = [r, h - r])
            cyl_y(r, d, x, 0, z);
}

module su_shell() {
    difference() {
        rounded_box_y(SU_W, SU_D, SU_H, SU_FILLET);

        // hollow it out, leaving the back open
        translate([SU_WALL, SU_WALL, SU_WALL])
            rounded_box_y(SU_W - 2 * SU_WALL, SU_D,
                          SU_H - 2 * SU_WALL, SU_FILLET - SU_WALL);

        cyl_y(FP_DIA / 2, THRU, CX, -7, FP_Z);
        box_at(LCD_W, THRU, LCD_H, CX - LCD_W / 2, -7, LCD_Z - LCD_H / 2);
        cyl_y(BTN_DIA / 2, THRU, SU_W * 0.28, -7, BTN_Z);
        cyl_y(LED_DIA / 2, THRU, SU_W * 0.72, -7, BTN_Z);
        cyl_z(GLAND_DIA / 2, THRU, CX, SU_D * 0.7, -7);
    }
}

module su_back_plate() {
    difference() {
        box_at(SU_W, SU_BACKPLATE_T, SU_H, 0, SU_D, 0);
        for (x = [14, SU_W - 14], z = [14, SU_H - 14])
            cyl_y(MOUNT_HOLE_DIA / 2, SU_BACKPLATE_T + 8, x, SU_D - 4, z);
    }
}

// The concealed component bay. Everything here mounts off the back plate, in
// the depth band behind the panel-mounted LCD and sensor, so nothing fouls.
module su_internals() {
    esp_x = 14;
    esp_z = 16;
    buck_x = SU_W - MOD_BUCK[0] - 16;
    term_z = SU_H - MOD_TERM[2] - 16;

    // ESP32-S3 on standoffs off the back plate
    color(C_MCU)
        box_at(MOD_ESP[0], MOD_ESP[1], MOD_ESP[2], esp_x, BAY_Y, esp_z);
    color(C_PLATE)
        for (dx = [6, MOD_ESP[0] - 6], dz = [5, MOD_ESP[2] - 5])
            cyl_y(STANDOFF_DIA / 2, STANDOFF_H,
                  esp_x + dx, BAY_Y + MOD_ESP[1], esp_z + dz);

    // buck converter beside it, terminal block up in the corner
    color(C_RELAY)
        box_at(MOD_BUCK[0], MOD_BUCK[1], MOD_BUCK[2],
               buck_x, BACK_IN - MOD_BUCK[1], esp_z);
    color(C_TERMINAL)
        box_at(MOD_TERM[0], MOD_TERM[1], MOD_TERM[2],
               esp_x, BACK_IN - MOD_TERM[1], term_z);

    // short loom from the terminal block down toward the board
    color(C_WIRE)
        cyl_z(3, 26, esp_x + 10, BACK_IN - 6, term_z - 26);
}

module scanner_head(show_back = true) {
    color(C_ENCL) su_shell();
    if (show_back) color(C_CARCASS) su_back_plate();

    // R307 fingerprint sensor: barrel behind the panel, face flush in the hole
    color(C_PLATE)  cyl_y(FP_BODY_DIA / 2, FP_BODY_D, CX, SU_WALL, FP_Z);
    color(C_SENSOR) cyl_y(FP_DIA / 2 - 0.6, SU_WALL + 1, CX, -0.5, FP_Z);

    // 16x2 LCD: glass in the window, full module PCB behind it
    color(C_LCD)
        box_at(LCD_W - 1, 2, LCD_H - 1,
               CX - (LCD_W - 1) / 2, 0.6, LCD_Z - (LCD_H - 1) / 2);
    color(C_BOARD)
        box_at(MOD_LCD[0], MOD_LCD[1], MOD_LCD[2],
               CX - MOD_LCD[0] / 2, SU_WALL, LCD_Z - MOD_LCD[2] / 2);

    color(C_BUTTON) cyl_y(BTN_DIA / 2 - 0.5, 8, SU_W * 0.28, -2, BTN_Z);
    color(C_LED)    cyl_y(LED_DIA / 2 - 0.4, 6, SU_W * 0.72, -1.5, BTN_Z);

    su_internals();
}

// Four-leg pedestal, centred on the origin in X and Y, floor at z = 0.
module scanner_stand() {
    leg_x = [-BASE_W / 2 + LEG_INSET, BASE_W / 2 - LEG_INSET - LEG];
    leg_y = [-BASE_D / 2 + LEG_INSET, BASE_D / 2 - LEG_INSET - LEG];
    inner_x0 = leg_x[0] + LEG;
    inner_y0 = leg_y[0] + LEG;
    rail_off = (LEG - RAIL) / 2;

    color(C_PLINTH)
        box_at(BASE_W, BASE_D, BASE_T, -BASE_W / 2, -BASE_D / 2, 0);

    color(C_HANDLE) {
        for (x = leg_x, y = leg_y)
            box_at(LEG, LEG, STAND_H - BASE_T, x, y, BASE_T);

        for (y = leg_y)
            box_at(leg_x[1] - inner_x0, RAIL, RAIL,
                   inner_x0, y + rail_off, RAIL_Z);
        for (x = leg_x)
            box_at(RAIL, leg_y[1] - inner_y0, RAIL,
                   x + rail_off, inner_y0, RAIL_Z);
    }

    color(C_PLATE)
        box_at(TOP_PLATE_W, TOP_PLATE_D, TOP_PLATE_T,
               -TOP_PLATE_W / 2, -TOP_PLATE_D / 2, STAND_H);

    // cable conduit down the back of the post
    color(C_WIRE)
        cyl_z(CONDUIT_DIA / 2, STAND_H - BASE_T, 0, leg_y[1] + LEG / 2, BASE_T);
}

module scanner_kiosk() {
    lift = SU_D * tan(HEAD_TILT);
    head_y = -30;

    scanner_stand();

    translate([-SU_W / 2, head_y, STAND_H + TOP_PLATE_T]) {
        color(C_PLATE) wedge(SU_W, SU_D, lift);

        // pivot on the bottom-back edge so the face tips up, ATM style
        translate([0, SU_D, 0])
            rotate([-HEAD_TILT, 0, 0])
                translate([0, -SU_D, 0])
                    scanner_head();
    }
}
