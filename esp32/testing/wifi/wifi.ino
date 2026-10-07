// TEST 9 of 12: joining the network.
//
// Wiring: none.
//
// Fill in the two lines below, upload, and watch it connect. Nothing else in
// the system works until this does, because both readers ask the server before
// they open anything.
//
// Expect: an IP address on the same subnet as the machine running the backend,
// and an RSSI better than -75 dBm where the reader will actually be mounted.
// Test it in the doorway, not on your desk. A stockroom door is usually a steel
// frame and it will cost you 10 to 20 dBm.
//
// The ESP32 radio is 2.4 GHz only. If your router publishes one name for both
// bands and hands out 5 GHz first, the join fails with no useful error. Ask for
// a 2.4 GHz name, or split the bands.
//
// Monitor commands:
//   s  scan and list every network in range, strongest first
//   c  connect
//   d  disconnect
//   i  what we are connected to
//   w  watch the signal for 30 seconds, so you can walk the room

#include <WiFi.h>

static const char* WIFI_SSID = "YOUR_WIFI";
static const char* WIFI_PASS = "YOUR_WIFI_PASSWORD";

static const uint8_t PIN_LED = 2;

const char* quality(int32_t rssi) {
    if (rssi >= -60) {
        return "strong";
    }
    if (rssi >= -70) {
        return "workable";
    }
    if (rssi >= -80) {
        return "weak, expect dropouts";
    }
    return "too weak to rely on";
}

const char* encryption(wifi_auth_mode_t mode) {
    switch (mode) {
        case WIFI_AUTH_OPEN:
            return "open";
        case WIFI_AUTH_WEP:
            return "WEP";
        case WIFI_AUTH_WPA_PSK:
            return "WPA";
        case WIFI_AUTH_WPA2_PSK:
            return "WPA2";
        case WIFI_AUTH_WPA_WPA2_PSK:
            return "WPA/WPA2";
        case WIFI_AUTH_WPA3_PSK:
            return "WPA3";
        case WIFI_AUTH_WPA2_WPA3_PSK:
            return "WPA2/WPA3";
        default:
            return "other";
    }
}

void scan() {
    Serial.println(F("scanning..."));
    const int16_t found = WiFi.scanNetworks();

    if (found <= 0) {
        Serial.println(F("  nothing in range. That usually means the antenna"));
        Serial.println(F("  end of the board is pressed against metal."));
        return;
    }

    Serial.print(F("  "));
    Serial.print(found);
    Serial.println(F(" networks:"));

    for (int16_t i = 0; i < found; i++) {
        Serial.print(F("  "));
        Serial.print(WiFi.RSSI(i));
        Serial.print(F(" dBm  ch"));
        Serial.print(WiFi.channel(i));
        Serial.print(F("  "));
        Serial.print(encryption(WiFi.encryptionType(i)));
        Serial.print(F("  "));
        Serial.print(WiFi.SSID(i));

        if (WiFi.SSID(i) == WIFI_SSID) {
            Serial.print(F("   <- the one you configured"));
        }
        Serial.println();
    }

    WiFi.scanDelete();
}

void showStatus() {
    if (WiFi.status() != WL_CONNECTED) {
        Serial.println(F("  not connected."));
        return;
    }

    Serial.print(F("  ssid    : "));
    Serial.println(WiFi.SSID());
    Serial.print(F("  ip      : "));
    Serial.println(WiFi.localIP());
    Serial.print(F("  gateway : "));
    Serial.println(WiFi.gatewayIP());
    Serial.print(F("  mask    : "));
    Serial.println(WiFi.subnetMask());
    Serial.print(F("  dns     : "));
    Serial.println(WiFi.dnsIP());
    Serial.print(F("  mac     : "));
    Serial.println(WiFi.macAddress());
    Serial.print(F("  channel : "));
    Serial.println(WiFi.channel());
    Serial.print(F("  signal  : "));
    Serial.print(WiFi.RSSI());
    Serial.print(F(" dBm, "));
    Serial.println(quality(WiFi.RSSI()));
    Serial.println();
    Serial.println(F("API_BASE in Config.h must point at a machine on this"));
    Serial.println(F("same subnet, by IP. A .local name will not resolve."));
}

void connect() {
    if (strcmp(WIFI_SSID, "YOUR_WIFI") == 0) {
        Serial.println(F("WIFI_SSID is still the placeholder. Edit the top of"));
        Serial.println(F("this sketch first."));
        return;
    }

    Serial.print(F("joining "));
    Serial.print(WIFI_SSID);
    Serial.println(F("..."));

    WiFi.mode(WIFI_STA);
    WiFi.begin(WIFI_SSID, WIFI_PASS);

    const uint32_t started = millis();
    while (WiFi.status() != WL_CONNECTED && millis() - started < 20000) {
        digitalWrite(PIN_LED, !digitalRead(PIN_LED));
        delay(250);
    }

    if (WiFi.status() == WL_CONNECTED) {
        digitalWrite(PIN_LED, HIGH);
        Serial.print(F("  connected in "));
        Serial.print(millis() - started);
        Serial.println(F(" ms"));
        showStatus();
        return;
    }

    digitalWrite(PIN_LED, LOW);
    Serial.println(F("  could not join. The usual causes:"));
    Serial.println(F("  1. the name is a 5 GHz band. The ESP32 is 2.4 GHz only."));
    Serial.println(F("  2. a typo in the password"));
    Serial.println(F("  3. the network needs a browser login. Those never work."));
    Serial.println(F("  4. MAC filtering on the router"));
    Serial.println(F("  Press 's' to scan, and check the name appears at all."));
}

void watchSignal() {
    if (WiFi.status() != WL_CONNECTED) {
        Serial.println(F("  connect first."));
        return;
    }

    Serial.println(F("30 seconds of signal. Carry the board to where it will be"));
    Serial.println(F("mounted and watch the number."));

    const uint32_t started = millis();
    while (millis() - started < 30000) {
        if (WiFi.status() != WL_CONNECTED) {
            Serial.println(F("  dropped out."));
            return;
        }
        const int32_t rssi = WiFi.RSSI();
        Serial.print(F("  "));
        Serial.print(rssi);
        Serial.print(F(" dBm  "));
        Serial.println(quality(rssi));
        delay(1000);
    }
    Serial.println(F("  done."));
}

void help() {
    Serial.println();
    Serial.println(F("s scan   c connect   d disconnect   i info   w watch signal"));
}

void setup() {
    Serial.begin(115200);
    delay(300);

    pinMode(PIN_LED, OUTPUT);
    digitalWrite(PIN_LED, LOW);

    Serial.println();
    Serial.println(F("WiFi test."));
    WiFi.mode(WIFI_STA);
    WiFi.disconnect();
    delay(100);

    scan();
    connect();
    help();
}

void loop() {
    if (!Serial.available()) {
        return;
    }

    const char c = Serial.read();
    switch (c) {
        case 's':
            scan();
            break;
        case 'c':
            connect();
            break;
        case 'd':
            WiFi.disconnect();
            digitalWrite(PIN_LED, LOW);
            Serial.println(F("  disconnected."));
            break;
        case 'i':
            showStatus();
            break;
        case 'w':
            watchSignal();
            break;
        case '\n':
        case '\r':
            break;
        default:
            help();
            break;
    }
}
