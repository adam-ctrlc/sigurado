#pragma once

#include <Arduino.h>

// DOOR node settings. Edit these, nothing else.
namespace cfg {

// Either the router, or the cabinet node's SHARE_SSID if this node is out of
// range of the router. See SHARE_WIFI in the cabinet config.
static const char* const WIFI_SSID = "YOUR_WIFI";
static const char* const WIFI_PASS = "YOUR_WIFI_PASSWORD";

// No trailing slash. LAN address of the machine running the Rust backend.
static const char* const API_BASE = "http://192.168.1.50:8080/api";

// From the web app: Admin > Devices. The secret is shown only once.
static const char* const DEVICE_ID     = "PASTE_DOOR_DEVICE_ID";
static const char* const DEVICE_SECRET = "PASTE_DOOR_DEVICE_SECRET";
static const char* const DEVICE_TAG    = "door";

// ---- pins: ESP32 DevKit v1, the 30-pin board ------------------------------
// GPIO 6 to 11 are wired to the flash chip and 34/35/36/39 are input only with
// no pull-up, so none of them appear here.
static const uint8_t PIN_FP_RX  = 16;   // UART2: sensor TX -> here
static const uint8_t PIN_FP_TX  = 17;   // UART2: sensor RX -> here
static const uint8_t PIN_SDA    = 21;
static const uint8_t PIN_SCL    = 22;
static const uint8_t PIN_BUTTON = 4;
static const uint8_t PIN_LED    = 2;
static const uint8_t PIN_RELAY  = 5;

static const uint8_t LCD_ADDRESS = 0x27;

// Set true only if an electric door strike is actually wired up.
static const bool RELAY_FITTED     = false;
static const bool RELAY_ACTIVE_LOW = true;   // most cheap relay boards are

static const uint32_t RELAY_HOLD_MS   = 3000;
static const uint32_t MESSAGE_HOLD_MS = 2500;
static const uint32_t HEARTBEAT_MS    = 30000;
static const uint32_t SESSION_HINT_MS = 120000;  // matches SESSION_TTL_SECS

}  // namespace cfg
