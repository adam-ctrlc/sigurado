#pragma once

#include <Arduino.h>

// CABINET node settings. Edit these, nothing else.
namespace cfg {

static const char* const WIFI_SSID = "YOUR_WIFI";
static const char* const WIFI_PASS = "YOUR_WIFI_PASSWORD";

// No trailing slash. LAN address of the machine running the Rust backend.
static const char* const API_BASE = "http://192.168.1.50:8080/api";

// From the web app: Admin > Devices. The secret is shown only once.
static const char* const DEVICE_ID     = "PASTE_CABINET_DEVICE_ID";
static const char* const DEVICE_SECRET = "PASTE_CABINET_DEVICE_SECRET";
static const char* const DEVICE_TAG    = "cabinet";

// ---- sharing WiFi with the other node -------------------------------------
// With this on, the cabinet joins the router AND runs its own small access
// point, passing traffic between the two. The door node then joins SHARE_SSID
// instead of the router and still reaches the server normally.
//
// Only turn it on if the door node cannot see the router from where it is
// mounted. It costs radio time and makes the door node depend on this one being
// awake, so two independent links are the better default when both can reach
// the router. Turn it on for ONE node only.
static const bool SHARE_WIFI = false;
static const char* const SHARE_SSID = "sigurado-link";
static const char* const SHARE_PASS = "change-this-please";   // 8 chars or more

// ---- pins: ESP32 DevKit v1, the 30-pin board ------------------------------
// GPIO 6 to 11 are wired to the flash chip and 34/35/36/39 are input only with
// no pull-up, so none of them appear here.
static const uint8_t PIN_FP_RX  = 16;   // UART2: sensor TX -> here
static const uint8_t PIN_FP_TX  = 17;   // UART2: sensor RX -> here
static const uint8_t PIN_SDA    = 21;
static const uint8_t PIN_SCL    = 22;
static const uint8_t PIN_BUTTON = 4;
static const uint8_t PIN_LED    = 2;
static const uint8_t PIN_RELAY  = 5;    // solenoid, through the relay contacts
static const uint8_t PIN_DOOR   = 18;   // limit switch, door-closed sensor
static const uint8_t PIN_BUZZER = 19;

// SIM800L on UART1. Its RX sits behind a divider or a level shifter: that line
// is not 3.3V tolerant, whatever the listing claims.
static const uint8_t PIN_GSM_RX  = 26;   // UART1: modem TX -> here
static const uint8_t PIN_GSM_TX  = 27;   // UART1: modem RX -> here
static const uint8_t PIN_GSM_RST = 25;   // 255 to leave the reset line unwired
static const uint32_t GSM_BAUD   = 9600;

static const uint8_t LCD_ADDRESS = 0x27;

static const bool RELAY_ACTIVE_LOW = true;   // most cheap relay boards are

// Fail-secure solenoid: energised only long enough to let the door open.
static const uint32_t UNLOCK_HOLD_MS  = 4000;
static const uint32_t MESSAGE_HOLD_MS = 2500;
static const uint32_t HEARTBEAT_MS    = 30000;

// The code has to stay on screen long enough to write down.
static const uint32_t CODE_HOLD_MS = 15000;

// Door-state sensing. A limit switch is a mechanical contact, so it needs
// debouncing, and the terminal you used decides the polarity: COM to GND with
// the normally-open terminal on the pin reads LOW when the door presses the
// lever. Wire the normally-closed terminal instead and set this false.
static const bool DOOR_CLOSED_WHEN_LOW = true;
static const uint32_t DOOR_DEBOUNCE_MS = 20;

// Door-left-open escalation.
static const uint32_t OPEN_WARN_MS  = 30000;
static const uint32_t OPEN_ALARM_MS = 60000;

// How often to drain the text queue, and how many to take per pass.
static const uint32_t SMS_POLL_MS = 15000;
static const size_t SMS_BATCH = 3;

}  // namespace cfg
