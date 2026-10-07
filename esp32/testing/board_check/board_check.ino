// TEST 1 of 12: the board itself.
//
// Nothing is wired up yet. Run this before anything else: it proves the cable,
// the driver, the upload path and the serial monitor all work, so that when a
// later test fails you know the fault is in the part you just added.
//
// Wiring: none. Just the USB cable.
// Expect: the blue LED next to the pin header blinks once a second, and a chip
// report appears in the monitor.

static const uint8_t PIN_LED = 2;

static const uint32_t BLINK_MS = 500;

uint32_t lastBlink = 0;
bool ledOn = false;

void report() {
    Serial.println();
    Serial.println(F("=== board ==="));
    Serial.print(F("chip        : "));
    Serial.println(ESP.getChipModel());
    Serial.print(F("revision    : "));
    Serial.println(ESP.getChipRevision());
    Serial.print(F("cores       : "));
    Serial.println(ESP.getChipCores());
    Serial.print(F("cpu freq    : "));
    Serial.print(getCpuFrequencyMhz());
    Serial.println(F(" MHz"));
    Serial.print(F("flash size  : "));
    Serial.print(ESP.getFlashChipSize() / 1024 / 1024);
    Serial.println(F(" MB"));
    Serial.print(F("sketch size : "));
    Serial.print(ESP.getSketchSize() / 1024);
    Serial.println(F(" kB"));
    Serial.print(F("free space  : "));
    Serial.print(ESP.getFreeSketchSpace() / 1024);
    Serial.println(F(" kB"));
    Serial.print(F("free heap   : "));
    Serial.print(ESP.getFreeHeap());
    Serial.println(F(" bytes"));
    // A DevKit v1 has no PSRAM. Zero here is correct, not a fault.
    Serial.print(F("psram       : "));
    Serial.println(ESP.getPsramSize());

    uint64_t mac = ESP.getEfuseMac();
    char macStr[18];
    snprintf(macStr, sizeof(macStr), "%02X:%02X:%02X:%02X:%02X:%02X",
             (uint8_t)(mac >> 40), (uint8_t)(mac >> 32), (uint8_t)(mac >> 24),
             (uint8_t)(mac >> 16), (uint8_t)(mac >> 8), (uint8_t)mac);
    Serial.print(F("mac         : "));
    Serial.println(macStr);

    Serial.println();
    Serial.println(F("The LED should be blinking. Type anything and press enter"));
    Serial.println(F("to prove the monitor can send as well as receive."));
}

void setup() {
    Serial.begin(115200);
    delay(300);

    pinMode(PIN_LED, OUTPUT);
    digitalWrite(PIN_LED, LOW);

    report();
}

void loop() {
    const uint32_t now = millis();

    if (now - lastBlink >= BLINK_MS) {
        lastBlink = now;
        ledOn = !ledOn;
        digitalWrite(PIN_LED, ledOn ? HIGH : LOW);
    }

    if (Serial.available()) {
        String line = Serial.readStringUntil('\n');
        line.trim();
        if (line.length() == 0) {
            return;
        }
        if (line == "r") {
            report();
            return;
        }
        Serial.print(F("heard: "));
        Serial.println(line);
        Serial.println(F("Both directions work. Move on to test 2, i2c_scan."));
    }
}
