// Sigurado 3D models: shared dimensions and colors.
// All units are millimetres.
//   X = width  (0 at the left)
//   Y = depth  (0 at the FRONT, increasing toward the back)
//   Z = height (0 at the floor)

// Curve quality. $fn is left at 0 so the adaptive pair below is used instead:
// $fa caps the angle per facet and $fs the facet length, so a big wire spool
// gets far more segments than a 5 mm LED. Raise HD for smoother, slower.
HD  = 1;              // 1 = smooth, 2 = very smooth, 0.5 = fast/draft
$fn = 0;
$fa = 3 / HD;         // degrees per facet
$fs = 0.7 / HD;       // mm per facet

// ---------------------------------------------------------------- cabinet --
// Prototype: ONE storage slot with one door, one solenoid, one limit switch.
// The overall height is derived from the slot, so changing SLOT_H resizes the
// whole cabinet. Sized to sit on a bench, not on the floor.
CABINET_W = 550;
CABINET_D = 400;
SLOT_H    = 260;   // interior height of the single storage slot

PANEL_T      = 18;
BACK_T       = 12;
PLINTH_H     = 60;
PLINTH_INSET = 30;

SHELF_INSET = 8;

ELEC_BAY_H      = 150;   // concealed service bay across the top
SERVICE_PANEL_T = 15;

DOOR_T        = 18;
DOOR_GAP      = 3;
DOOR_OPEN_DEG = 150;

HANDLE_LEN = 120;
HANDLE_DIA = 14;

LOCK_W = 55; LOCK_D = 40; LOCK_H = 30;
LOCK_BOLT_DIA = 10;
LOCK_BOLT_LEN = 22;

NODE_W = 110; NODE_H = 85; NODE_D = 36;

// Door-state sensing: lever limit switch on the carcass, pressed by a striker
// pad on the door. Without this the server cannot tell "locked" from "bolt shot
// into thin air". Body and lever are sized from a V-153-1C25 style microswitch.
SWITCH_W = 20; SWITCH_D = 11; SWITCH_H = 11;
LEVER_D  = 22; LEVER_W = 6; LEVER_T = 1.5;
STRIKER_W = 16; STRIKER_D = 8; STRIKER_H = 16;

// Latch tab on the door. The far end is ramped so the closing door cams the
// spring bolt up and it drops into the hole on its own: no power needed.
TAB_T      = 6;
TAB_FLAT_D = 30;
TAB_RAMP_D = 20;

PSU      = [110, 62, 40];
RELAY    = [78, 55, 20];
TERMINAL = [85, 26, 26];
MCU      = [63, 25.5, 7];

// -------------------------------------------------- derived cabinet values --
// Stack, bottom to top: plinth -> slot -> divider -> service bay -> top panel.
INTERIOR_X0 = PANEL_T;
INTERIOR_W  = CABINET_W - 2 * PANEL_T;
INTERIOR_D  = CABINET_D - BACK_T;

INTERIOR_Z0 = PLINTH_H + PANEL_T;         // floor of the slot
DIVIDER_Z   = INTERIOR_Z0 + SLOT_H;       // underside of the service divider
ELEC_Z0     = DIVIDER_Z + PANEL_T;        // floor of the service bay
INTERIOR_Z1 = ELEC_Z0 + ELEC_BAY_H;
CABINET_H   = INTERIOR_Z1 + PANEL_T;

SHELF_Z = [];   // no shelves: the prototype is a single open slot

DOOR_COUNT = 1;
DOOR_H     = ELEC_Z0 - PLINTH_H - DOOR_GAP;
DOOR_Z     = [PLINTH_H];

SENSE_Z = INTERIOR_Z0 + SLOT_H * 0.65;    // switch and striker height

// ---------------------------------------------------------- scanner unit --
// Sized from the real modules below, not guessed. The old 150 x 115 x 55 box
// could not hold a 16x2 LCD module (80 x 36, and 26 deep with its I2C backpack)
// and an ESP32 with pin headers at the same time, so it grew.
SU_W = 160; SU_H = 160; SU_D = 70;
SU_WALL         = 3;
SU_FILLET       = 8;
SU_BACKPLATE_T  = 3;

// Real module footprints, [width, depth, height] as mounted.
MOD_LCD  = [80, 26, 36];    // LCD1602 + I2C backpack
MOD_ESP  = [70, 20, 30];    // ESP32-S3 DevKitC, headers included
MOD_BUCK = [45, 16, 22];    // 12V -> 5V buck converter
MOD_TERM = [45, 12, 14];    // screw terminal block

FP_DIA      = 30;     // R307 face, i.e. the panel cutout
FP_BODY_DIA = 21;     // barrel behind the panel
FP_BODY_D   = 25;
FP_Z_FRAC   = 0.40;

LCD_W      = 71.5;    // viewing area, i.e. the panel window
LCD_H      = 24.5;
LCD_Z_FRAC = 0.72;

BTN_DIA    = 12;
LED_DIA    = 5;
BTN_Z_FRAC = 0.13;
GLAND_DIA  = 12;
MOUNT_HOLE_DIA = 4.5;

STANDOFF_DIA = 6;
STANDOFF_H   = 8;

// ------------------------------------------------- floor stand (4 legs) --
STAND_H   = 1150;     // top of the legs
LEG       = 30;
BASE_W    = 260;
BASE_D    = 220;
BASE_T    = 12;
LEG_INSET = 14;
TOP_PLATE_W = 190;
TOP_PLATE_D = 100;
TOP_PLATE_T = 12;
RAIL        = 20;
RAIL_Z      = 170;
CONDUIT_DIA = 18;
HEAD_TILT   = 15;     // degrees, tips the face up toward the user

// ------------------------------------------------- construction / framing --
// Sizes for the "how to build it" views.
LUMBER  = 38;   // nominal 2x2 lumber, dressed (38 x 38 mm)
SKIN_T  = 12;   // plywood skin: sides, back, top, doors
SHELF_T = 18;   // plywood shelves and the service divider
PLY_ALPHA = 0.40;
EXPLODE   = 90; // how far the doors and panels are pulled off in build views

C_LUMBER = [0.62, 0.42, 0.24];
C_PLY    = [0.86, 0.72, 0.48];

// ----------------------------------------------------------- presentation --
SHOW_LABELS = true;
LABEL_SIZE  = 20;
LABEL_DEPTH = 1.5;
LABEL_FONT  = "Arial";

C_CARCASS  = [0.80, 0.78, 0.74];
C_SHELF    = [0.70, 0.68, 0.65];
C_BACK     = [0.66, 0.64, 0.61];
C_PLINTH   = [0.42, 0.42, 0.44];
C_DOOR     = [0.60, 0.73, 0.88];
C_HANDLE   = [0.55, 0.57, 0.60];
C_LOCK     = [0.85, 0.22, 0.20];
C_BOLT     = [0.75, 0.75, 0.78];
C_ENCL     = [0.16, 0.42, 0.75];
C_SENSOR   = [0.25, 0.62, 0.90];
C_LCD      = [0.10, 0.32, 0.22];
C_BUTTON   = [0.95, 0.75, 0.20];
C_LED      = [0.30, 0.85, 0.40];
C_BOARD    = [0.15, 0.45, 0.30];
C_MCU      = [0.55, 0.20, 0.55];
C_LABEL    = [0.15, 0.15, 0.17];
C_SERVICE  = [0.34, 0.36, 0.40];
C_PSU      = [0.30, 0.32, 0.36];
C_RELAY    = [0.20, 0.24, 0.30];
C_TERMINAL = [0.72, 0.68, 0.30];
C_PLATE    = [0.58, 0.60, 0.63];
C_WIRE     = [0.25, 0.25, 0.28];
C_SWITCH   = [0.14, 0.14, 0.16];
C_STRIKER  = [0.82, 0.82, 0.86];

C_MAT_COMPONENTS = [0.30, 0.70, 0.45];
C_MAT_INSTRUMENTS = [0.95, 0.62, 0.20];
C_MAT_BULK        = [0.45, 0.60, 0.70];

SLOT_LABEL = "LAB MATERIALS";
