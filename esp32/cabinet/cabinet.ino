/*
  Sigurado - CABINET node (storage box scanner)

  Scanning here unlocks the solenoid ONLY IF this same user already scanned at
  the door and their session is still alive. A registered user who skipped the
  door is refused and logged as a tailgater.

  When it does unlock, the server hands back a short code. It goes on the LCD and
  nowhere else, and the person types it on the website to record what they took.
  That is what ties a written record to a real opening.

  A limit switch reports whether the door is really shut, so the audit trail can
  never claim "secured" while the bolt sits in thin air.

  This node also carries the SIM800L, so it is the one that sends the text
  alerts the server queues.

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
      R307/AS608   VCC -> 3V3     TX -> GPIO16   RX -> GPIO17   GND -> GND
      16x2 I2C     SDA -> GPIO21  SCL -> GPIO22  VCC -> 5V      GND -> GND
      Button       GPIO4  -> button -> GND        (internal pull-up)
      Status LED   GPIO2  -> 220R -> LED -> GND
      Relay IN     GPIO5           (relay logic 5V; solenoid on its own 12V)
      Limit switch GPIO18 -> COM,  NO -> GND      (internal pull-up)
      Buzzer +     GPIO19          (active buzzer)
      SIM800L      TX -> GPIO26    RX <- GPIO27 through a divider
                   RST -> GPIO25   VCC -> its OWN 4V supply, GND common

  The SIM800L pulls up to 2A in bursts. On the ESP32 regulator it browns out
  mid-message and looks like a software fault, so give it a separate supply and
  a large capacitor across the module.

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
#include "src/lock/DoorSensor.h"
#include "src/lock/Solenoid.h"
#include "src/net/Net.h"
#include "src/sms/Gsm.h"
#include "src/ui/Panel.h"

HardwareSerial fpSerial(2);
HardwareSerial gsmSerial(1);

Net net(cfg::WIFI_SSID, cfg::WIFI_PASS, cfg::API_BASE,
        cfg::DEVICE_ID, cfg::DEVICE_SECRET);
ApiClient api(net, cfg::DEVICE_TAG);
Biometric bio(fpSerial);
Panel panel(cfg::LCD_ADDRESS, cfg::PIN_LED, cfg::PIN_BUTTON, cfg::PIN_BUZZER);
EnrollFlow enroll(api, bio, panel);
Solenoid solenoid(cfg::PIN_RELAY, cfg::RELAY_ACTIVE_LOW, cfg::UNLOCK_HOLD_MS);
DoorSensor door(cfg::PIN_DOOR, cfg::OPEN_WARN_MS, cfg::OPEN_ALARM_MS,
                cfg::DOOR_CLOSED_WHEN_LOW, cfg::DOOR_DEBOUNCE_MS);
Gsm gsm(gsmSerial, cfg::PIN_GSM_RX, cfg::PIN_GSM_TX, cfg::PIN_GSM_RST);
Diag diag;

uint32_t lastHeartbeat = 0;
uint32_t lastSmsPoll = 0;
uint32_t smsWaiting = 0;
bool smsMine = false;      // the server says this is the node with the modem

void showIdle() {
    panel.showIfChanged("Sigurado box",
                        door.isClosed() ? "Scan finger" : "Door is open");
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
        panel.show("Unlocked", res.userName);
        panel.led(true);
        panel.chirp(60);
        solenoid.release();
        panel.led(false);

        // The code is the point of the unlock: without it they cannot record
        // what they took, so it gets the screen to itself.
        if (res.checkoutCode[0] != '\0') {
            panel.show("Your code", res.checkoutCode);
            panel.chirp(40);
            delay(cfg::CODE_HOLD_MS);
            panel.show("Type it on the", "checkout page");
            delay(cfg::MESSAGE_HOLD_MS);
        }
        panel.show("Take what you", "need, then close");
        delay(cfg::MESSAGE_HOLD_MS);
        return;
    }

    if (res.enrollHint) {
        panel.show("Not registered", "Press enroll");
        panel.blink(2);
    } else if (strcmp(res.reason, "no_active_session") == 0) {
        panel.show("Still locked", "Scan door first");
        panel.blink(4);
    } else if (strcmp(res.reason, "session_expired") == 0) {
        panel.show("Still locked", "Scan door again");
        panel.blink(4);
    } else {
        char pretty[28];
        strncpy(pretty, res.reason, sizeof(pretty) - 1);
        pretty[sizeof(pretty) - 1] = '\0';
        for (char* p = pretty; *p; p++) {
            if (*p == '_') *p = ' ';
        }
        panel.show("Still locked", pretty);
        panel.blink(4);
    }
    delay(cfg::MESSAGE_HOLD_MS);
}

// Reminds anyone nearby while the door is left open, and tells the server.
void watchDoor() {
    switch (door.update()) {
        case DoorEvent::JustClosed: {
            panel.buzzer(false);
            const HeartbeatResult beat = api.heartbeat(true);
            if (beat.ok) {
                smsMine = beat.smsCapable;
                smsWaiting = beat.pendingSms;
            }
            panel.show("Door closed", "Thank you");
            delay(1200);
            showIdle();
            break;
        }

        case DoorEvent::Warning:
            panel.showf("Please close it", "open for %lus",
                        (unsigned long)(door.openForMs() / 1000));
            panel.chirp(60);
            delay(400);
            break;

        case DoorEvent::Alarm:
            panel.showIfChanged("Door is open", "Please close it");
            panel.buzzer(true);
            break;

        case DoorEvent::NoChange:
            break;
    }
}

// Sends whatever the server has queued. Only runs while nobody is at the
// reader: a SIM800L takes seconds per message and blocks while it works.
void drainSms() {
    if (!smsMine || !gsm.ready() || smsWaiting == 0) return;

    SmsJob jobs[cfg::SMS_BATCH];
    const int count = api.pendingSms(jobs, cfg::SMS_BATCH);
    if (count <= 0) {
        smsWaiting = 0;
        return;
    }

    panel.show("Sending alert", "please wait");
    for (int i = 0; i < count; i++) {
        char why[40] = {0};
        const bool sent = gsm.send(jobs[i].number, jobs[i].body, why,
                                   sizeof(why));
        api.reportSms(jobs[i].id, sent, why);
        Serial.printf("[sms] %s -> %s %s\n", jobs[i].number,
                      sent ? "sent" : "failed", sent ? "" : why);
    }
    smsWaiting = ((uint32_t)count < smsWaiting) ? smsWaiting - count : 0;
    showIdle();
}

void setup() {
    Serial.begin(115200);
    diag.begin();

    panel.begin(cfg::PIN_SDA, cfg::PIN_SCL);
    panel.show("Sigurado", "starting...");
    solenoid.begin();
    door.begin();

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
        delay(1200);

        if (cfg::SHARE_WIFI) {
            if (net.startBridge(cfg::SHARE_SSID, cfg::SHARE_PASS)) {
                panel.show("Sharing WiFi as", cfg::SHARE_SSID);
            } else {
                panel.show("Could not share", "carrying on");
            }
            delay(1500);
        }
    } else {
        panel.show("No WiFi yet", "check settings");
        delay(1500);
    }

    panel.show("Starting modem", "please wait");
    if (gsm.begin(cfg::GSM_BAUD)) {
        panel.showf("Modem ready", "signal %d/31", gsm.signal());
    } else {
        // Not fatal: the cabinet still works, the texts simply stay queued.
        panel.show("No modem found", "alerts will wait");
    }
    delay(1500);

    const HeartbeatResult beat = api.heartbeat(door.isClosed());
    smsMine = beat.smsCapable;
    smsWaiting = beat.pendingSms;

    showIdle();
}

void loop() {
    diag.loop();
    watchDoor();

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
            case FingerScan::Error:
                break;
        }
    }

    if (millis() - lastHeartbeat > cfg::HEARTBEAT_MS) {
        lastHeartbeat = millis();
        const HeartbeatResult beat = api.heartbeat(door.isClosed());
        if (beat.ok) {
            smsMine = beat.smsCapable;
            smsWaiting = beat.pendingSms;
        }
    }

    // Last, and only while the room is quiet, so sending never delays an unlock.
    if (millis() - lastSmsPoll > cfg::SMS_POLL_MS &&
        enroll.stage() == EnrollFlow::Stage::Idle && door.isClosed()) {
        lastSmsPoll = millis();
        drainSms();
    }

    delay(50);
}
