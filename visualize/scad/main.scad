-// Sigurado: open this file in OpenSCAD and press F5 to preview (F6 to render).
//
// Pick what to show with the `part` dropdown below, or use
// Window > Customizer in the OpenSCAD GUI.
//
//   all   = finished cabinet and kiosk, each beside its build view
//   build = just the two build views (2x2 lumber skeleton + plywood)

include <config.scad>
use <cabinet.scad>
use <scanner_unit.scad>

use <construction.scad>

part = "all"; // [all, build, cabinet_open, cabinet_closed, cabinet_service, cabinet_frame, kiosk, kiosk_frame, head, head_open]

GAP           = 340;
CAB_FRAME_X   = CABINET_W + GAP;
KIOSK_X       = CAB_FRAME_X + CABINET_W + 460;
KIOSK_FRAME_X = KIOSK_X + 440;
KIOSK_Y       = CABINET_D / 2;

if (part == "all") {
    cabinet(open_doors = true);
    translate([CAB_FRAME_X, 0, 0]) cabinet_frame();
    translate([KIOSK_X, KIOSK_Y, 0]) scanner_kiosk();
    translate([KIOSK_FRAME_X, KIOSK_Y, 0]) kiosk_frame();

} else if (part == "build") {
    cabinet_frame();
    translate([CABINET_W + 420, KIOSK_Y, 0]) kiosk_frame();

} else if (part == "cabinet_open") {
    cabinet(open_doors = true);

} else if (part == "cabinet_closed") {
    cabinet(open_doors = false);

} else if (part == "cabinet_service") {
    // service panel removed, exposing the concealed electronics bay
    cabinet(open_doors = true, service_open = true);

} else if (part == "cabinet_frame") {
    cabinet_frame();

} else if (part == "kiosk") {
    scanner_kiosk();

} else if (part == "kiosk_frame") {
    kiosk_frame();

} else if (part == "head") {
    scanner_head();

} else if (part == "head_open") {
    // back plate off: check the components actually fit and plan the wiring
    scanner_head(show_back = false);
}
