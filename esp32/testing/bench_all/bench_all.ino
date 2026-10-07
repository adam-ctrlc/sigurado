// TEST 12 of 12: everything at once.
//
// Run this last, on a node that is fully wired, right before you screw the
// enclosure shut. It checks every part in turn and prints one table saying what
// passed and what did not, so there is a single thing to photograph for the
// documentation.
//
// Set NODE below to match the board you are testing. The cabinet has three
// parts the door does not: the limit switch, the buzzer and the modem, and they
// are skipped rather than failed on a door node.
//
// Fill in the network and device settings the same way as tests 9 and 10.
//
// Monitor commands:
//   r  run every check again
//   l  live view: the fingerprint sensor and the door switch, continuously

#include <Adafruit_Fingerprint.h>
#include <HTTPClient.h>
#include <LiquidCrystal_I2C.h>
#include <WiFi.h>
#include <Wire.h>

#include "bench_all.h"

// "door" or "cabinet".
static const char* NODE = "cabinet";

static const char* WIFI_SSID = "YOUR_WIFI";
static const char* WIFI_PASS = "YOUR_WIFI_PASSWORD";
static const char* API_BASE = "http://192.168.1.50:8080/api";
static const char* DEVICE_ID = "PASTE_DEVICE_ID";
static const char* DEVICE_SECRET = "PASTE_DEVICE_SECRET";

static const uint8_t PIN_FP_RX = 16;
static const uint8_t PIN_FP_TX = 17;
static const uint8_t PIN_SDA = 21;
static const uint8_t PIN_SCL = 22;
static const uint8_t PIN_BUTTON = 4;
static const uint8_t PIN_LED = 2;
static const uint8_t PIN_RELAY = 5;
static const uint8_t PIN_DOOR = 18;
static const uint8_t PIN_BUZZER = 19;
static const uint8_t PIN_GSM_RX = 26;
static const uint8_t PIN_GSM_TX = 27;

static const uint8_t LCD_ADDRESS = 0x27;
static const bool RELAY_ACTIVE_LOW = true;

HardwareSerial fpSerial(2);
HardwareSerial gsmSerial(1);
Adafruit_Fingerprint finger(&fpSerial);
LiquidCrystal_I2C lcd(LCD_ADDRESS, 20, 4);

bool isCabinet() {
    return strcmp(NODE, "cabinet") == 0;
}

static const uint8_t MAX_CHECKS = 12;
Check checks[MAX_CHECKS];
uint8_t checkCount = 0;

void record(const char* name, Outcome outcome, const char* detail) {
    if (checkCount >= MAX_CHECKS) {
        return;
    }
    checks[checkCount].name = name;
    checks[checkCount].outcome = outcome;
    snprintf(checks[checkCount].detail, sizeof(checks[checkCount].detail), "%s", detail);
    checkCount++;

    Serial.print(F("  "));
    switch (outcome) {
        case Outcome::Pass:
            Serial.print(F("[ ok ] "));
            break;
        case Outcome::Fail:
            Serial.print(F("[fail] "));
            break;
        case Outcome::Skip:
            Serial.print(F("[skip] "));
            break;
        case Outcome::Warn:
            Serial.print(F("[warn] "));
            break;
    }
    Serial.print(name);
    Serial.print(F(": "));
    Serial.println(detail);
}

void checkI2c() {
    Wire.begin(PIN_SDA, PIN_SCL);
    Wire.setClock(100000);

    uint8_t foundAt = 0;
    for (uint8_t address = 1; address < 127; address++) {
        Wire.beginTransmission(address);
        if (Wire.endTransmission() == 0) {
            foundAt = address;
            break;
        }
    }

    char detail[64];
    if (foundAt == 0) {
        record("i2c bus", Outcome::Fail, "nothing answered on 21/22");
        return;
    }

    snprintf(detail, sizeof(detail), "device at 0x%02X", foundAt);
    if (foundAt == LCD_ADDRESS) {
        record("i2c bus", Outcome::Pass, detail);
    } else {
        snprintf(detail, sizeof(detail), "found 0x%02X, config says 0x%02X",
                 foundAt, LCD_ADDRESS);
        record("i2c bus", Outcome::Warn, detail);
    }
}

void checkLcd() {
    lcd.init();
    lcd.backlight();
    lcd.clear();
    lcd.setCursor(0, 0);
    lcd.print("Sigurado bench test");
    lcd.setCursor(0, 1);
    lcd.print(NODE);
    lcd.setCursor(0, 2);
    lcd.print("all four rows should");
    lcd.setCursor(0, 3);
    lcd.print("have text on them");
    record("display", Outcome::Pass, "wrote four rows, look at the glass");
}

void checkFingerprint() {
    fpSerial.begin(57600, SERIAL_8N1, PIN_FP_RX, PIN_FP_TX);
    delay(100);
    finger.begin(57600);
    delay(100);

    if (!finger.verifyPassword()) {
        record("fingerprint", Outcome::Fail, "no answer, check 16/17 and 3V3");
        return;
    }

    finger.getTemplateCount();
    char detail[64];
    snprintf(detail, sizeof(detail), "%u template(s) stored", finger.templateCount);
    record("fingerprint", Outcome::Pass, detail);
}

void checkButton() {
    pinMode(PIN_BUTTON, INPUT_PULLUP);
    delay(10);

    if (digitalRead(PIN_BUTTON) == LOW) {
        record("button", Outcome::Warn, "reads pressed while idle, check the legs");
    } else {
        record("button", Outcome::Pass, "idle high, press it in the live view");
    }
}

void checkRelay() {
    digitalWrite(PIN_RELAY, RELAY_ACTIVE_LOW ? HIGH : LOW);
    pinMode(PIN_RELAY, OUTPUT);
    digitalWrite(PIN_RELAY, RELAY_ACTIVE_LOW ? HIGH : LOW);

    // One short click, long enough to hear, too short to move a bolt far.
    digitalWrite(PIN_RELAY, RELAY_ACTIVE_LOW ? LOW : HIGH);
    delay(200);
    digitalWrite(PIN_RELAY, RELAY_ACTIVE_LOW ? HIGH : LOW);

    record("relay", Outcome::Pass, "clicked once, released. Did you hear it?");
}

void checkDoorSwitch() {
    if (!isCabinet()) {
        record("limit switch", Outcome::Skip, "door node has no cabinet sensor");
        return;
    }

    pinMode(PIN_DOOR, INPUT_PULLUP);
    delay(10);

    const bool level = digitalRead(PIN_DOOR);
    char detail[64];
    snprintf(detail, sizeof(detail), "reads %s right now",
             level == LOW ? "LOW (shut)" : "HIGH (open)");
    record("limit switch", Outcome::Pass, detail);
}

void checkBuzzer() {
    if (!isCabinet()) {
        record("buzzer", Outcome::Skip, "door node has no buzzer");
        return;
    }

    ledcAttach(PIN_BUZZER, 2000, 10);
    ledcWriteTone(PIN_BUZZER, 1200);
    delay(120);
    ledcWriteTone(PIN_BUZZER, 1800);
    delay(160);
    ledcWriteTone(PIN_BUZZER, 0);
    ledcDetach(PIN_BUZZER);

    record("buzzer", Outcome::Pass, "two notes played. Did you hear them?");
}

void checkWifi() {
    if (strcmp(WIFI_SSID, "YOUR_WIFI") == 0) {
        record("wifi", Outcome::Skip, "ssid not filled in");
        return;
    }

    WiFi.mode(WIFI_STA);
    WiFi.begin(WIFI_SSID, WIFI_PASS);

    const uint32_t started = millis();
    while (WiFi.status() != WL_CONNECTED && millis() - started < 20000) {
        delay(250);
    }

    if (WiFi.status() != WL_CONNECTED) {
        record("wifi", Outcome::Fail, "could not join, 2.4 GHz only remember");
        return;
    }

    const int32_t rssi = WiFi.RSSI();
    char detail[64];
    snprintf(detail, sizeof(detail), "%s at %d dBm",
             WiFi.localIP().toString().c_str(), (int)rssi);

    record("wifi", rssi < -78 ? Outcome::Warn : Outcome::Pass, detail);
}

void checkBackend() {
    if (WiFi.status() != WL_CONNECTED) {
        record("backend", Outcome::Skip, "no network");
        return;
    }
    if (strcmp(DEVICE_ID, "PASTE_DEVICE_ID") == 0) {
        record("backend", Outcome::Skip, "device id not filled in");
        return;
    }

    char url[160];
    snprintf(url, sizeof(url), "%s/device/heartbeat", API_BASE);

    HTTPClient http;
    http.setTimeout(8000);
    http.begin(url);
    http.addHeader("Content-Type", "application/json");
    http.addHeader("X-Device-Id", DEVICE_ID);
    http.addHeader("X-Device-Secret", DEVICE_SECRET);

    const char* body = "{\"firmware\":\"bench-all\"}";
    const int code = http.POST((uint8_t*)body, strlen(body));
    http.end();

    char detail[64];
    snprintf(detail, sizeof(detail), "heartbeat returned %d", code);

    if (code == 200) {
        record("backend", Outcome::Pass, detail);
    } else if (code == 401) {
        record("backend", Outcome::Fail, "401, the device id or secret is wrong");
    } else if (code < 0) {
        record("backend", Outcome::Fail, "no answer, check the IP and the firewall");
    } else {
        record("backend", Outcome::Fail, detail);
    }
}

void checkGsm() {
    if (!isCabinet()) {
        record("sim800l", Outcome::Skip, "door node has no modem");
        return;
    }

    gsmSerial.begin(9600, SERIAL_8N1, PIN_GSM_RX, PIN_GSM_TX);
    delay(500);

    gsmSerial.println("AT");

    String buffer;
    const uint32_t started = millis();
    while (millis() - started < 3000) {
        while (gsmSerial.available()) {
            buffer += (char)gsmSerial.read();
        }
        if (buffer.indexOf("OK") >= 0) {
            record("sim800l", Outcome::Pass, "answered AT, run test 11 for signal");
            return;
        }
        delay(10);
    }

    record("sim800l", Outcome::Fail, "silent, almost always the 4V supply");
}

void checkMemory() {
    const size_t freeHeap = ESP.getFreeHeap();
    const size_t largest = ESP.getMaxAllocHeap();

    char detail[64];
    snprintf(detail, sizeof(detail), "%u free, %u largest block",
             (unsigned)freeHeap, (unsigned)largest);

    // The firmware reboots itself below 24k free or an 8k largest block, so
    // anything near that on a bare bench is a problem worth knowing about now.
    record("memory", (freeHeap < 40000 || largest < 16000) ? Outcome::Warn : Outcome::Pass, detail);
}

void summary() {
    uint8_t passed = 0;
    uint8_t failed = 0;
    uint8_t warned = 0;
    uint8_t skipped = 0;

    Serial.println();
    Serial.println(F("================ bench summary ================"));
    Serial.print(F("node: "));
    Serial.println(NODE);
    Serial.println();

    for (uint8_t i = 0; i < checkCount; i++) {
        switch (checks[i].outcome) {
            case Outcome::Pass:
                Serial.print(F("  ok    "));
                passed++;
                break;
            case Outcome::Fail:
                Serial.print(F("  FAIL  "));
                failed++;
                break;
            case Outcome::Warn:
                Serial.print(F("  warn  "));
                warned++;
                break;
            case Outcome::Skip:
                Serial.print(F("  skip  "));
                skipped++;
                break;
        }
        Serial.print(checks[i].name);
        for (uint8_t pad = strlen(checks[i].name); pad < 14; pad++) {
            Serial.print(' ');
        }
        Serial.println(checks[i].detail);
    }

    Serial.println();
    Serial.print(F("  "));
    Serial.print(passed);
    Serial.print(F(" ok, "));
    Serial.print(failed);
    Serial.print(F(" failed, "));
    Serial.print(warned);
    Serial.print(F(" warnings, "));
    Serial.print(skipped);
    Serial.println(F(" skipped"));
    Serial.println(F("==============================================="));

    if (failed == 0) {
        Serial.println(F("Ready for the real firmware. Copy your settings into"));
        Serial.println(F("Config.h and upload the door or cabinet sketch."));
    } else {
        Serial.println(F("Fix the failures first. Each one has its own test"));
        Serial.println(F("sketch in this folder with the wiring at the top."));
    }

    lcd.clear();
    lcd.setCursor(0, 0);
    lcd.print(failed == 0 ? "Bench passed" : "Bench failed");
    lcd.setCursor(0, 1);
    lcd.print(NODE);
    char line[21];
    snprintf(line, sizeof(line), "%u ok, %u failed", passed, failed);
    lcd.setCursor(0, 2);
    lcd.print(line);
    snprintf(line, sizeof(line), "%u warn, %u skip", warned, skipped);
    lcd.setCursor(0, 3);
    lcd.print(line);
}

void runAll() {
    checkCount = 0;

    Serial.println();
    Serial.print(F("Running every check for the "));
    Serial.print(NODE);
    Serial.println(F(" node..."));
    Serial.println();

    checkI2c();
    checkLcd();
    checkFingerprint();
    checkButton();
    checkRelay();
    checkDoorSwitch();
    checkBuzzer();
    checkGsm();
    checkWifi();
    checkBackend();
    checkMemory();

    summary();
}

void liveView() {
    Serial.println();
    Serial.println(F("Live view for 60 seconds. Press a finger, press the button,"));
    Serial.println(F("open and close the door. Anything sent stops it early."));

    while (Serial.available()) {
        Serial.read();
    }

    const uint32_t started = millis();
    while (millis() - started < 60000 && !Serial.available()) {
        const uint8_t image = finger.getImage();
        if (image == FINGERPRINT_OK) {
            if (finger.image2Tz() == FINGERPRINT_OK) {
                if (finger.fingerFastSearch() == FINGERPRINT_OK) {
                    Serial.print(F("  finger: slot "));
                    Serial.print(finger.fingerID);
                    Serial.print(F(", confidence "));
                    Serial.println(finger.confidence);
                } else {
                    Serial.println(F("  finger: not recognized"));
                }
            }
            while (finger.getImage() != FINGERPRINT_NOFINGER) {
                delay(50);
            }
        }

        static bool lastButton = HIGH;
        const bool button = digitalRead(PIN_BUTTON);
        if (button != lastButton) {
            lastButton = button;
            Serial.println(button == LOW ? F("  button: pressed") : F("  button: released"));
            delay(30);
        }

        if (isCabinet()) {
            static bool lastDoor = HIGH;
            const bool door = digitalRead(PIN_DOOR);
            if (door != lastDoor) {
                lastDoor = door;
                Serial.println(door == LOW ? F("  door: shut") : F("  door: open"));
                delay(30);
            }
        }

        delay(20);
    }

    while (Serial.available()) {
        Serial.read();
    }
    Serial.println(F("  live view ended."));
}

void setup() {
    Serial.begin(115200);
    delay(500);

    pinMode(PIN_LED, OUTPUT);
    digitalWrite(PIN_LED, HIGH);

    Serial.println();
    Serial.println(F("Sigurado bench test."));
    runAll();

    Serial.println();
    Serial.println(F("r run again   l live view"));
}

void loop() {
    if (!Serial.available()) {
        return;
    }

    const char c = Serial.read();
    if (c == 'r') {
        runAll();
    } else if (c == 'l') {
        liveView();
    }
}
