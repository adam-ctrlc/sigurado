// TEST 4 of 12: the enroll button and the status LED.
//
// Wiring, push button:
//   one leg to GPIO 4
//   the other leg to GND
// No resistor. The internal pull-up is switched on below, so the pin idles HIGH
// and reads LOW while the button is held.
//
// The LED on GPIO 2 is the blue one already soldered to the board. Nothing to
// wire for it.
//
// Expect: the LED follows the button, and every press, release and long press
// is named in the monitor with how long it lasted.
//
// A four-legged tactile switch has two pairs that are permanently joined. If
// the pin reads LOW forever you picked a joined pair: move one wire to the leg
// diagonally opposite.

static const uint8_t PIN_BUTTON = 4;
static const uint8_t PIN_LED = 2;

static const uint32_t DEBOUNCE_MS = 30;
static const uint32_t LONG_PRESS_MS = 1200;

bool stableState = HIGH;
bool lastReading = HIGH;
uint32_t lastChange = 0;
uint32_t pressedAt = 0;
bool longFired = false;
uint32_t pressCount = 0;

void setup() {
    Serial.begin(115200);
    delay(300);

    pinMode(PIN_BUTTON, INPUT_PULLUP);
    pinMode(PIN_LED, OUTPUT);
    digitalWrite(PIN_LED, LOW);

    Serial.println();
    Serial.println(F("Button and LED test."));
    Serial.println(F("Idle reads HIGH. Holding the button reads LOW."));
    Serial.print(F("Right now the pin reads: "));
    Serial.println(digitalRead(PIN_BUTTON) == HIGH ? F("HIGH (good, not pressed)")
                                                   : F("LOW (stuck, or held down)"));
    Serial.println();
}

void loop() {
    const bool reading = digitalRead(PIN_BUTTON);
    const uint32_t now = millis();

    if (reading != lastReading) {
        lastReading = reading;
        lastChange = now;
    }

    // Only believe a level that has held steady past the bounce window.
    if (now - lastChange >= DEBOUNCE_MS && reading != stableState) {
        stableState = reading;

        if (stableState == LOW) {
            pressedAt = now;
            longFired = false;
            pressCount++;
            digitalWrite(PIN_LED, HIGH);
            Serial.print(F("press   #"));
            Serial.println(pressCount);
        } else {
            digitalWrite(PIN_LED, LOW);
            Serial.print(F("release after "));
            Serial.print(now - pressedAt);
            Serial.println(F(" ms"));
        }
    }

    if (stableState == LOW && !longFired && now - pressedAt >= LONG_PRESS_MS) {
        longFired = true;
        Serial.println(F("long press. The firmware uses this to start enrollment."));
        for (uint8_t i = 0; i < 4; i++) {
            digitalWrite(PIN_LED, LOW);
            delay(60);
            digitalWrite(PIN_LED, HIGH);
            delay(60);
        }
    }
}
