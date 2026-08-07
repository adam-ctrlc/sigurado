/*
  Sigurado - DOOR node (entry scanner)

  Scanning here opens a timed access session for that user. The cabinet will
  only unlock for the same person while that session is alive, which is what
  blocks tailgating.

  Board:  ESP32 Dev Module, the 30-pin DevKit v1 (ESP32-WROOM-32)
  Board settings in the IDE (Tools menu), and what they cost us:
      Board            ESP32 Dev Module
      PSRAM            Disabled      the 30-pin DevKit v1 has no PSRAM chip, and
                                     nothing here allocates from it
      JTAG Adapter     Disabled      which leaves GPIO 12 to 15 free; the pin map
                                     below avoids them anyway
      Partition Scheme Default 4MB with spiffs (1.2MB APP)
      Flash Size       4MB

  Libraries (Arduino Library Manager):
      Adafruit Fingerprint Sensor Library
      LiquidCrystal I2C   (johnrickman)
      ArduinoJson         (v7)

  Wiring
      R307/AS608  VCC -> 3V3     TX -> GPIO16   RX -> GPIO17   GND -> GND
      16x2 I2C    SDA -> GPIO21  SCL -> GPIO22  VCC -> 5V      GND -> GND
      Button      GPIO4 -> button -> GND        (internal pull-up)
      Status LED  GPIO2 -> 220R -> LED -> GND
      Relay IN    GPIO5          (optional door strike, own 12V supply)

  If this node cannot see the router from the doorway, set SHARE_WIFI on the
  cabinet node and point WIFI_SSID here at its SHARE_SSID. Nothing else changes:
  the cabinet passes this node's traffic through to the server.

  Built for continuous running: no Arduino String anywhere on the hot path, all
  results land in fixed buffers, and Diag reboots the node if the heap ever
  fragments past the point of safety.

  Settings live in src/Config.h. Logic lives in src/ by domain.
*/

#include "src/Config.h"
#include "src/api/ApiClient.h"
#include "src/biometric/Biometric.h"
#include "src/diag/Diag.h"
#include "src/enroll/EnrollFlow.h"
#include "src/net/Net.h"
#include "src/ui/Panel.h"

HardwareSerial fpSerial(2);

Net net(cfg::WIFI_SSID, cfg::WIFI_PASS, cfg::API_BASE,
        cfg::DEVICE_ID, cfg::DEVICE_SECRET);
ApiClient api(net, cfg::DEVICE_TAG);
Biometric bio(fpSerial);
Panel panel(cfg::LCD_ADDRESS, cfg::PIN_LED, cfg::PIN_BUTTON);
EnrollFlow enroll(api, bio, panel);
Diag diag;

uint32_t lastHeartbeat = 0;
uint32_t sessionHintUntil = 0;   // display only; the server is authoritative

void fireRelay() {
    if (!cfg::RELAY_FITTED) return;
    digitalWrite(cfg::PIN_RELAY, cfg::RELAY_ACTIVE_LOW ? LOW : HIGH);
    delay(cfg::RELAY_HOLD_MS);
    digitalWrite(cfg::PIN_RELAY, cfg::RELAY_ACTIVE_LOW ? HIGH : LOW);
}

void showIdle() {
    if (sessionHintUntil > millis()) {
        char line[17];
        snprintf(line, sizeof(line), "session %lus",
                 (unsigned long)((sessionHintUntil - millis()) / 1000));
        panel.showIfChanged("Sigurado door", line);
    } else {
        panel.showIfChanged("Sigurado door", "Scan finger");
    }
}

void handleMatch(uint16_t slot) {
    panel.show("Checking...", "one moment");

    char token[32];
    api.tokenFor(slot, token, sizeof(token));
    const ScanResult res = api.scan(token);

    if (!res.ok) {
        panel.showf("Server not ready", "code %d", res.httpCode);
        panel.blink(3);
        delay(cfg::MESSAGE_HOLD_MS);
        return;
    }

    if (res.unlock) {
        panel.show("Welcome", res.userName);
        panel.led(true);
        fireRelay();
        panel.led(false);
        sessionHintUntil = millis() + cfg::SESSION_HINT_MS;
        delay(cfg::MESSAGE_HOLD_MS);
        return;
    }

    if (res.enrollHint) {
        panel.show("Not registered", "Press enroll");
        panel.blink(2);
    } else {
        char pretty[28];
        strncpy(pretty, res.reason, sizeof(pretty) - 1);
        pretty[sizeof(pretty) - 1] = '\0';
        for (char* p = pretty; *p; p++) {
            if (*p == '_') *p = ' ';
        }
        panel.show("Not open yet", pretty);
        panel.blink(4);
    }
    delay(cfg::MESSAGE_HOLD_MS);
}

void setup() {
    Serial.begin(115200);
    diag.begin();

    if (cfg::RELAY_FITTED) {
        pinMode(cfg::PIN_RELAY, OUTPUT);
        digitalWrite(cfg::PIN_RELAY, cfg::RELAY_ACTIVE_LOW ? HIGH : LOW);
    }

    panel.begin(cfg::PIN_SDA, cfg::PIN_SCL);
    panel.show("Sigurado", "starting...");

    if (!bio.begin(cfg::PIN_FP_RX, cfg::PIN_FP_TX)) {
        panel.show("Sensor not found", "check wiring");
        while (true) delay(1000);
    }
    Serial.printf("templates stored: %u\n", bio.templateCount());

    panel.show("Joining WiFi", cfg::WIFI_SSID);
    if (net.connect()) {
        net.syncTime();
        char addr[20];
        net.ip(addr, sizeof(addr));
        panel.show("Connected", addr);
    } else {
        panel.show("No WiFi yet", "check settings");
    }
    delay(1500);
    showIdle();
}

void loop() {
    diag.loop();

    if (panel.buttonPressed()) {
        enroll.onButton();
        if (enroll.stage() == EnrollFlow::Stage::Idle) showIdle();
        return;
    }

    if (enroll.stage() == EnrollFlow::Stage::Idle) {
        switch (bio.poll()) {
            case FingerScan::Matched:
                handleMatch(bio.lastSlot());
                bio.waitRemoved(4000);
                showIdle();
                break;

            case FingerScan::NoMatch:
                panel.show("Not registered", "Press enroll");
                panel.blink(2);
                delay(cfg::MESSAGE_HOLD_MS);
                bio.waitRemoved(4000);
                showIdle();
                break;

            case FingerScan::None:
                showIdle();   // cheap: skips the write unless the text changed
                break;

            case FingerScan::Error:
                break;
        }
    }

    if (millis() - lastHeartbeat > cfg::HEARTBEAT_MS) {
        lastHeartbeat = millis();
        api.heartbeat();
    }

    delay(50);
}
