// TEST 6 of 12: the limit switch that senses whether the cabinet door is shut.
// Cabinet node only.
//
// Wiring:
//   COM to GND
//   NO (normally open) to GPIO 18
// The internal pull-up is on, so the pin idles HIGH and goes LOW when the door
// presses the lever. If you wired NC instead, the reading is inverted and this
// sketch will tell you so.
//
// Mount it so the door presses the lever when shut, not when open. A switch
// that only makes contact at the very end of the travel will chatter every time
// somebody leans on the cabinet.
//
// Expect: a line every time the state settles, with the bounce count. A good
// microswitch bounces two or three times over a few milliseconds. Twenty
// bounces means a loose lever or a bad solder joint.
//
// What to write into Config.h, from the summary this prints:
//   DOOR_CLOSED_WHEN_LOW = true   if the pin reads LOW with the door shut
//   DOOR_CLOSED_WHEN_LOW = false  if it reads HIGH with the door shut

static const uint8_t PIN_DOOR = 18;
static const uint8_t PIN_LED = 2;

static const uint32_t DEBOUNCE_MS = 20;

bool stableState = HIGH;
bool lastReading = HIGH;
uint32_t lastChange = 0;
uint32_t settledAt = 0;
uint32_t bounces = 0;
uint32_t transitions = 0;

// Filled in by the calibration step so the summary can name the right setting.
bool calibrated = false;
bool closedIsLow = true;

void printState(bool level) {
    Serial.print(F("pin reads "));
    Serial.print(level == LOW ? F("LOW ") : F("HIGH"));
    if (calibrated) {
        const bool closed = (level == LOW) == closedIsLow;
        Serial.print(closed ? F("  door SHUT") : F("  door OPEN"));
    }
}

void calibrate() {
    Serial.println();
    Serial.println(F("Hold the door shut, or press the lever, then press enter."));

    while (Serial.available()) {
        Serial.read();
    }
    while (!Serial.available()) {
        delay(20);
    }
    while (Serial.available()) {
        Serial.read();
    }

    delay(DEBOUNCE_MS * 2);
    const bool shutLevel = digitalRead(PIN_DOOR);
    closedIsLow = (shutLevel == LOW);
    calibrated = true;

    Serial.print(F("With the door shut the pin reads "));
    Serial.println(shutLevel == LOW ? F("LOW.") : F("HIGH."));
    Serial.print(F("So set  DOOR_CLOSED_WHEN_LOW = "));
    Serial.println(closedIsLow ? F("true;") : F("false;"));

    if (!closedIsLow) {
        Serial.println(F("That is the NC terminal. It works, but NO is the safer"));
        Serial.println(F("choice: a cut wire then reads as an open door rather"));
        Serial.println(F("than a shut one, so a fault cannot hide an open cabinet."));
    }
    Serial.println();
    Serial.println(F("Now open and close it a few times."));
}

void setup() {
    Serial.begin(115200);
    delay(300);

    pinMode(PIN_DOOR, INPUT_PULLUP);
    pinMode(PIN_LED, OUTPUT);

    stableState = digitalRead(PIN_DOOR);
    lastReading = stableState;

    Serial.println();
    Serial.println(F("Limit switch test."));
    Serial.print(F("At rest: "));
    printState(stableState);
    Serial.println();
    Serial.println(F("Press enter to calibrate, or just start moving the door."));

    calibrate();
    settledAt = millis();
}

void loop() {
    const bool reading = digitalRead(PIN_DOOR);
    const uint32_t now = millis();

    if (reading != lastReading) {
        lastReading = reading;
        lastChange = now;
        bounces++;
    }

    if (now - lastChange >= DEBOUNCE_MS && reading != stableState) {
        stableState = reading;
        transitions++;

        digitalWrite(PIN_LED, stableState == LOW ? HIGH : LOW);

        Serial.print(F("#"));
        Serial.print(transitions);
        Serial.print(F("  "));
        printState(stableState);
        Serial.print(F("  after "));
        Serial.print(now - settledAt);
        Serial.print(F(" ms, "));
        Serial.print(bounces);
        Serial.println(bounces == 1 ? F(" bounce") : F(" bounces"));

        if (bounces > 12) {
            Serial.println(F("  that is a lot of chatter. Check the lever and the"));
            Serial.println(F("  solder joints before trusting this reading."));
        }

        bounces = 0;
        settledAt = now;
    }

    if (Serial.available()) {
        while (Serial.available()) {
            Serial.read();
        }
        calibrate();
    }
}
