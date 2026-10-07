// TEST 7 of 12: the relay, and through it the solenoid lock or door strike.
//
// Read this before wiring anything.
//
// The solenoid does NOT run off the ESP32. It needs its own 12V supply, and the
// relay contacts sit between that supply and the coil. The only wires between
// the ESP32 and the relay board are 5V, GND and the signal on GPIO 5. Powering
// a solenoid from the board's regulator browns it out and reboots it mid-scan.
//
// Wiring, ESP32 to relay board:
//   VIN  to relay VCC
//   GND  to relay GND
//   GPIO 5 to relay IN
//
// Wiring, 12V side:
//   12V positive to relay COM
//   relay NO to solenoid positive
//   solenoid negative to 12V ground
//   a flyback diode (1N4007) across the solenoid coil, band to the positive
//   side. Without it the collapsing coil throws a spike that resets the ESP32
//   or welds the relay contacts shut.
//   the 12V ground and the ESP32 GND joined at one point.
//
// Run it the first time with the 12V supply switched OFF. You will hear the
// relay click and see its LED. Only once the clicks land in the right places
// should you power the coil.
//
// Expect: a click on lock and another on unlock. The firmware is fail secure,
// so the coil is energized only for the few seconds the door may be opened, and
// it is never left on.
//
// Monitor commands:
//   u  unlock for the real hold time, then relock
//   1  energize and leave it (be careful, most coils are not rated continuous)
//   0  de-energize
//   i  invert the polarity and say what to put in Config.h
//   t  click ten times, to check for a sticking contact

static const uint8_t PIN_RELAY = 5;
static const uint8_t PIN_LED = 2;

// Most of the cheap blue relay boards pull IN low to switch on.
bool activeLow = true;

static const uint32_t UNLOCK_HOLD_MS = 4000;

bool energized = false;

void driveRelay(bool on) {
    energized = on;
    const uint8_t level = (on == activeLow) ? LOW : HIGH;
    digitalWrite(PIN_RELAY, level);
    digitalWrite(PIN_LED, on ? HIGH : LOW);

    Serial.print(F("  coil "));
    Serial.print(on ? F("ON  ") : F("OFF "));
    Serial.print(F("(pin driven "));
    Serial.print(level == LOW ? F("LOW") : F("HIGH"));
    Serial.println(F(")"));
}

void help() {
    Serial.println();
    Serial.println(F("u unlock cycle   1 on   0 off   i invert   t ten clicks"));
    Serial.print(F("polarity now: RELAY_ACTIVE_LOW = "));
    Serial.println(activeLow ? F("true") : F("false"));
}

void unlockCycle() {
    Serial.println(F("unlocking..."));
    driveRelay(true);

    const uint32_t started = millis();
    while (millis() - started < UNLOCK_HOLD_MS) {
        delay(100);
    }

    driveRelay(false);
    Serial.println(F("relocked. That is exactly what a good scan does."));
}

void tenClicks() {
    Serial.println(F("ten cycles, listening for a contact that fails to release."));
    for (uint8_t i = 1; i <= 10; i++) {
        driveRelay(true);
        delay(250);
        driveRelay(false);
        delay(250);
        Serial.print(F("  "));
        Serial.println(i);
    }
    Serial.println(F("If any click sounded weak or was missed, replace the board."));
}

void setup() {
    Serial.begin(115200);
    delay(300);

    pinMode(PIN_LED, OUTPUT);

    // Drive the pin to the safe level BEFORE making it an output, otherwise the
    // brief float on reset can flick the lock open.
    digitalWrite(PIN_RELAY, activeLow ? HIGH : LOW);
    pinMode(PIN_RELAY, OUTPUT);
    digitalWrite(PIN_RELAY, activeLow ? HIGH : LOW);
    energized = false;

    Serial.println();
    Serial.println(F("Relay and lock test."));
    Serial.println(F("Start with the 12V supply OFF and just listen for clicks."));
    Serial.println(F("The relay should be silent right now. If it clicked on"));
    Serial.println(F("reset, press 'i' to flip the polarity."));
    help();
}

void loop() {
    if (!Serial.available()) {
        return;
    }

    const char c = Serial.read();
    switch (c) {
        case 'u':
            unlockCycle();
            break;
        case '1':
            Serial.println(F("holding the coil on. Press 0 before it gets warm."));
            driveRelay(true);
            break;
        case '0':
            driveRelay(false);
            break;
        case 'i':
            activeLow = !activeLow;
            digitalWrite(PIN_RELAY, activeLow ? HIGH : LOW);
            energized = false;
            Serial.print(F("  put this in Config.h: RELAY_ACTIVE_LOW = "));
            Serial.println(activeLow ? F("true;") : F("false;"));
            Serial.println(F("  the relay should now be released and silent."));
            break;
        case 't':
            tenClicks();
            break;
        case '\n':
        case '\r':
            break;
        default:
            help();
            break;
    }
}
