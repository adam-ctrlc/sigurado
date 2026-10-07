// TEST 5 of 12: the AS608 fingerprint sensor, with the display alongside it.
//
// This is the pair the reader is actually made of, on one board: a finger goes
// on the sensor and the person is told what happened on the glass. Running them
// together is the only way to catch a message that reads fine in the monitor
// and badly on the display, which is the whole point of doing it before the
// real firmware goes on.
//
// Wiring, AS608 to ESP32 30-pin DevKit v1:
//
//   red          VCC, 3.3V             3V3
//   black        GND                   GND
//   yellow       TXD, sensor out       GPIO 16  (RX2)
//   white/brown  RXD, sensor in        GPIO 17  (TX2)
//   blue         WAKEUP, touch out     GPIO 4   (optional)
//   green        3.3VT, touch supply   3V3      (only with the wakeup wire)
//
// Wiring, 20x4 display backpack:
//
//   VCC          VIN   (5V. It will not light properly on 3.3V)
//   GND          GND
//   SDA          GPIO 21
//   SCL          GPIO 22
//
// The two do not fight over anything: the sensor has UART2 to itself and the
// display has the I2C bus to itself. They only share power and ground.
//
// The sensor is the 3.3V module, so its red wire goes to 3V3 while the display
// goes to VIN. Do not put the sensor on VIN. VIN is 5V straight off the USB
// socket and it will damage a 3.3V AS608. The R307 sold under the same
// description IS a 5V part, so check which one you have first.
//
// The sensor's data pair crosses over: its transmit line reaches 16, its
// receive line reaches 17. Straight through is the most common mistake and it
// fails silently, so if the handshake never answers, swap those two first.
//
// The display is optional here. Leave it off and everything still works, just
// on the monitor alone. Run test 3, lcd, on its own first if the glass stays
// blank: it hunts for the address and checks the contrast, which this does not.
//
// GPIO 4 IS ALREADY THE ENROLL BUTTON in door/src/Config.h and
// cabinet/src/Config.h. That is not a problem for this sketch, which has no
// button, but the real firmware cannot have both on one pin. Move the button,
// or leave the wakeup wire off.
//
// Expect: a handshake, the number of templates already stored, and every prompt
// and result on both the monitor and the glass.
//
// Monitor commands:
//   s  search: place a finger, get back the slot it matches
//   e  enroll: place the same finger twice, stored in the next free slot
//   d  delete one slot
//   c  count templates
//   l  list which slots are taken
//   t  test the touch wire and work out its polarity
//   x  empty the whole library (asks first)
//
// Slot numbers matter beyond this test. The firmware turns a slot into a token
// like "door:12" or "cabinet:12" and that is what the backend stores against a
// person, so a slot emptied here orphans that person's finger on the server.

#include <Adafruit_Fingerprint.h>
#include <LiquidCrystal_I2C.h>
#include <Wire.h>

static const uint8_t PIN_FP_RX = 16;
static const uint8_t PIN_FP_TX = 17;
static const uint32_t FP_BAUD = 57600;

static const uint8_t PIN_SDA = 21;
static const uint8_t PIN_SCL = 22;

// The two addresses these backpacks ship on. Both are tried at startup.
static const uint8_t LCD_ADDRESS = 0x27;
static const uint8_t LCD_ADDRESS_ALT = 0x3F;

// A 20x4 module. The backpack is the same PCF8574 either way, so the address
// says nothing about the size: a 16x2 answers on 0x27 as well.
static const uint8_t LCD_COLS = 20;
static const uint8_t LCD_ROWS = 4;

// One row plus the terminator. Every message buffer here is this wide.
static const size_t LINE_SIZE = LCD_COLS + 1;

// The blue wire. Set this true once it and the green one are connected.
static const bool TOUCH_WIRED = false;
static const uint8_t PIN_FP_TOUCH = 4;

// Most AS608 modules drive this high while a finger is on the ring. Press 't'
// to find out which way round yours is.
static bool touchActiveHigh = true;

// The AS608 library holds 127 on most modules, more on a few.
static const uint16_t MAX_SLOT = 127;

HardwareSerial fpSerial(2);
Adafruit_Fingerprint finger(&fpSerial);
LiquidCrystal_I2C lcd(LCD_ADDRESS, LCD_COLS, LCD_ROWS);

bool online = false;
bool lcdPresent = false;

// ---------------------------------------------------------------- display --

bool answersAt(uint8_t address) {
    Wire.beginTransmission(address);
    return Wire.endTransmission() == 0;
}

void startLcd() {
    Wire.begin(PIN_SDA, PIN_SCL);
    // 100 kHz. The cheap backpacks are unreliable at 400 kHz on long leads.
    Wire.setClock(100000);

    uint8_t address = 0;
    if (answersAt(LCD_ADDRESS)) {
        address = LCD_ADDRESS;
    } else if (answersAt(LCD_ADDRESS_ALT)) {
        address = LCD_ADDRESS_ALT;
    }

    if (address == 0) {
        lcdPresent = false;
        Serial.println(F("no display on the bus. Carrying on without it."));
        Serial.println(F("  If one is connected, run test 2 (i2c_scan) to find"));
        Serial.println(F("  what address it is really on."));
        return;
    }

    lcd = LiquidCrystal_I2C(address, LCD_COLS, LCD_ROWS);
    lcd.init();
    lcd.backlight();
    lcdPresent = true;

    Serial.print(F("display found at 0x"));
    Serial.print(address, HEX);
    Serial.print(F(", driving it as "));
    Serial.print(LCD_COLS);
    Serial.print('x');
    Serial.println(LCD_ROWS);

    if (address != LCD_ADDRESS) {
        Serial.print(F("  note: that is not 0x"));
        Serial.print(LCD_ADDRESS, HEX);
        Serial.println(F(". Put it in Config.h as LCD_ADDRESS."));
    }
}

// Writes all four rows. Every prompt goes to the monitor as well, so the two
// can never drift apart, and anything wider than the glass is called out here
// rather than quietly falling off the end.
void showAll(const char* l1, const char* l2, const char* l3, const char* l4) {
    const char* rows[4] = {l1, l2, l3, l4};

    Serial.print(F("  ["));
    bool first = true;
    for (uint8_t i = 0; i < LCD_ROWS; i++) {
        if (rows[i][0] == '\0') {
            continue;
        }
        if (!first) {
            Serial.print(F(" / "));
        }
        Serial.print(rows[i]);
        first = false;
    }
    Serial.println(F("]"));

    for (uint8_t i = 0; i < LCD_ROWS; i++) {
        if (strlen(rows[i]) > LCD_COLS) {
            Serial.print(F("  WARNING: row "));
            Serial.print(i + 1);
            Serial.print(F(" is "));
            Serial.print(strlen(rows[i]));
            Serial.print(F(" characters and the glass is "));
            Serial.print(LCD_COLS);
            Serial.println(F(" wide. It will be cut off."));
        }
    }

    if (!lcdPresent) {
        return;
    }

    lcd.clear();
    for (uint8_t i = 0; i < LCD_ROWS; i++) {
        if (rows[i][0] == '\0') {
            continue;
        }
        lcd.setCursor(0, i);
        lcd.print(rows[i]);
    }
}

// Two lines, sat in the middle of the four so they do not look stranded at the
// top of the display.
void show(const char* top, const char* bottom) {
    showAll("", top, bottom, "");
}

void idle() {
    showAll("SIGURADO", "", "Place your finger", "on the sensor");
}

// ---------------------------------------------------------------- sensor ---

void help() {
    Serial.println();
    Serial.println(F("s search   e enroll   d delete   c count   l list   t touch   x empty"));
}

bool connect() {
    fpSerial.begin(FP_BAUD, SERIAL_8N1, PIN_FP_RX, PIN_FP_TX);
    delay(100);
    finger.begin(FP_BAUD);
    delay(100);

    if (!finger.verifyPassword()) {
        Serial.println(F("no answer from the sensor."));
        Serial.println(F("  1. red on 3V3. On VIN it is 5V and this is a 3.3V part."));
        Serial.println(F("  2. black to GND, shared with the board"));
        Serial.println(F("  3. yellow to 16 and white to 17, crossed over"));
        Serial.println(F("  4. try swapping 16 and 17"));
        showAll("Sensor offline", "", "Check the wiring", "and the 3V3 line");
        return false;
    }

    Serial.println(F("sensor answered."));
    finger.getParameters();
    Serial.print(F("  capacity     : "));
    Serial.println(finger.capacity);
    Serial.print(F("  security     : "));
    Serial.println(finger.security_level);
    Serial.print(F("  baud         : "));
    Serial.println(finger.baud_rate);

    finger.getTemplateCount();
    Serial.print(F("  templates    : "));
    Serial.println(finger.templateCount);

    Serial.print(F("  touch wire   : "));
    Serial.println(TOUCH_WIRED ? F("in use on GPIO 4") : F("not wired, polling instead"));

    char stored[LINE_SIZE];
    char capacity[LINE_SIZE];
    snprintf(stored, sizeof(stored), "%u finger(s) stored", finger.templateCount);
    snprintf(capacity, sizeof(capacity), "room for %u", finger.capacity);
    showAll("Sensor ready", "", stored, capacity);
    return true;
}

bool touched() {
    if (!TOUCH_WIRED) {
        return false;
    }
    return digitalRead(PIN_FP_TOUCH) == (touchActiveHigh ? HIGH : LOW);
}

// Blocks until a finger is present or the wait runs out.
//
// With the touch wire connected this waits on the ring first, which lets the
// sensor stay dark until somebody actually reaches for it. Without it, the only
// way to notice a finger is to keep asking for an image, which lights the ring
// continuously.
bool waitForFinger(const char* title, const char* prompt, uint32_t timeoutMs) {
    showAll(title, "", prompt, "");
    const uint32_t started = millis();

    if (TOUCH_WIRED) {
        while (millis() - started < timeoutMs) {
            if (touched()) {
                break;
            }
            delay(20);
        }
        if (!touched()) {
            Serial.println(F("  timed out. Nothing touched the ring."));
            show("Timed out", "Please try again");
            return false;
        }
        // Settle: the image is smeared if it is taken as the finger lands.
        delay(50);
    }

    while (millis() - started < timeoutMs) {
        const uint8_t p = finger.getImage();
        if (p == FINGERPRINT_OK) {
            return true;
        }
        if (p != FINGERPRINT_NOFINGER) {
            Serial.print(F("  read error 0x"));
            Serial.println(p, HEX);
        }
        delay(50);
    }

    Serial.println(F("  timed out."));
    show("Timed out", "Please try again");
    return false;
}

void waitForRemoval() {
    show("Thank you", "Lift your finger");
    while (finger.getImage() != FINGERPRINT_NOFINGER) {
        delay(50);
    }
}

void doSearch() {
    if (!waitForFinger("Searching", "Place your finger", 10000)) {
        return;
    }

    uint8_t p = finger.image2Tz();
    if (p != FINGERPRINT_OK) {
        Serial.print(F("  could not read that print, error 0x"));
        Serial.println(p, HEX);
        Serial.println(F("  a dry or dirty finger is the usual cause."));
        showAll("Could not read that", "", "Please try again", "");
        waitForRemoval();
        idle();
        return;
    }

    p = finger.fingerFastSearch();
    char detail[LINE_SIZE];

    if (p == FINGERPRINT_OK) {
        Serial.print(F("  matched slot "));
        Serial.print(finger.fingerID);
        Serial.print(F(" with confidence "));
        Serial.println(finger.confidence);

        snprintf(detail, sizeof(detail), "slot %u, conf %u",
                 finger.fingerID, finger.confidence);

        // The firmware ignores anything under 50: a weak match on a stockroom
        // door is worse than asking the person to scan again.
        if (finger.confidence < 50) {
            Serial.println(F("  the firmware would refuse this. Too weak."));
            showAll("Match too weak", "", detail, "The door would stay shut");
        } else {
            showAll("Welcome back", "", detail, "");
        }
    } else if (p == FINGERPRINT_NOTFOUND) {
        Serial.println(F("  no match. Enroll it with 'e' first."));
        showAll("Not recognized", "", "Please try again, or", "enroll with 'e'");
    } else {
        Serial.print(F("  search error 0x"));
        Serial.println(p, HEX);
        snprintf(detail, sizeof(detail), "error 0x%02X", p);
        showAll("Sensor problem", "", detail, "");
    }

    delay(1500);
    waitForRemoval();
    idle();
}

int16_t nextFreeSlot() {
    for (uint16_t slot = 1; slot <= MAX_SLOT; slot++) {
        if (finger.loadModel(slot) != FINGERPRINT_OK) {
            return slot;
        }
    }
    return -1;
}

void doEnroll() {
    const int16_t slot = nextFreeSlot();
    if (slot < 0) {
        Serial.println(F("  the library is full."));
        showAll("The sensor is full", "", "Delete a slot first", "");
        return;
    }

    Serial.print(F("enrolling into slot "));
    Serial.println(slot);

    char title[LINE_SIZE];
    snprintf(title, sizeof(title), "Enrolling slot %d", slot);

    if (!waitForFinger(title, "Place your finger", 15000)) {
        idle();
        return;
    }
    if (finger.image2Tz(1) != FINGERPRINT_OK) {
        Serial.println(F("  first read failed. Start again."));
        showAll("First read was bad", "", "Start again", "");
        waitForRemoval();
        idle();
        return;
    }
    waitForRemoval();

    if (!waitForFinger(title, "Same finger again", 15000)) {
        idle();
        return;
    }
    if (finger.image2Tz(2) != FINGERPRINT_OK) {
        Serial.println(F("  second read failed. Start again."));
        showAll("Second read was bad", "", "Start again", "");
        waitForRemoval();
        idle();
        return;
    }

    const uint8_t made = finger.createModel();
    if (made == FINGERPRINT_ENROLLMISMATCH) {
        Serial.println(F("  the two reads did not match. Same finger both times."));
        showAll("Those did not match", "", "Use the same finger", "both times");
        waitForRemoval();
        idle();
        return;
    }
    if (made != FINGERPRINT_OK) {
        Serial.print(F("  could not build a template, error 0x"));
        Serial.println(made, HEX);
        showAll("Could not save that", "", "Please try again", "");
        waitForRemoval();
        idle();
        return;
    }

    if (finger.storeModel(slot) == FINGERPRINT_OK) {
        Serial.print(F("  stored in slot "));
        Serial.println(slot);
        Serial.print(F("  the firmware would call this door:"));
        Serial.print(slot);
        Serial.println(F(" (or cabinet:N on the other node)"));

        char saved[LINE_SIZE];
        char token[LINE_SIZE];
        snprintf(saved, sizeof(saved), "Saved to slot %d", slot);
        snprintf(token, sizeof(token), "known as door:%d", slot);
        showAll("You are all set", "", saved, token);
    } else {
        Serial.println(F("  storing failed."));
        showAll("Could not save that", "", "Please try again", "");
    }

    delay(2000);
    waitForRemoval();
    idle();
}

void doDelete() {
    Serial.println(F("slot number to delete, then enter:"));
    showAll("Delete a slot", "", "Type the number", "in the monitor");

    const uint32_t started = millis();
    while (!Serial.available() && millis() - started < 15000) {
        delay(20);
    }
    if (!Serial.available()) {
        Serial.println(F("  cancelled."));
        idle();
        return;
    }

    const long slot = Serial.parseInt();
    if (slot < 1 || slot > MAX_SLOT) {
        Serial.println(F("  out of range."));
        showAll("Not a slot number", "", "1 to 127 only", "");
        delay(1500);
        idle();
        return;
    }

    char detail[LINE_SIZE];
    if (finger.deleteModel(slot) == FINGERPRINT_OK) {
        Serial.print(F("  slot "));
        Serial.print(slot);
        Serial.println(F(" cleared. Whoever was bound to it can no longer get in."));
        snprintf(detail, sizeof(detail), "slot %ld is now free", slot);
        showAll("Deleted", "", detail, "That finger is out");
    } else {
        Serial.println(F("  delete failed. Perhaps it was already empty."));
        showAll("Delete failed", "", "It may already", "have been empty");
    }

    delay(1800);
    idle();
}

void doList() {
    Serial.println(F("occupied slots:"));
    uint16_t used = 0;
    uint16_t firstUsed = 0;
    uint16_t lastUsed = 0;

    for (uint16_t slot = 1; slot <= MAX_SLOT; slot++) {
        if (finger.loadModel(slot) == FINGERPRINT_OK) {
            Serial.print(F("  "));
            Serial.println(slot);
            if (used == 0) {
                firstUsed = slot;
            }
            lastUsed = slot;
            used++;
        }
    }
    if (used == 0) {
        Serial.println(F("  none."));
    }

    char count[LINE_SIZE];
    char span[LINE_SIZE];
    snprintf(count, sizeof(count), "%u slot(s) in use", used);
    if (used == 0) {
        snprintf(span, sizeof(span), "nothing enrolled");
    } else {
        snprintf(span, sizeof(span), "from %u to %u", firstUsed, lastUsed);
    }
    showAll("Stored fingers", "", count, span);

    delay(2000);
    idle();
}

// Works out whether the blue wire is connected and which way it swings. An
// unwired pin cannot be told from an idle one by looking, so this asks for a
// touch and watches for the change.
void doTouch() {
    Serial.println();
    Serial.println(F("Touch test on GPIO 4."));

    if (!TOUCH_WIRED) {
        Serial.println(F("  TOUCH_WIRED is false, so nothing uses this yet."));
        Serial.println(F("  Testing anyway. Set it true at the top of the sketch"));
        Serial.println(F("  once this passes."));
    }

    // Pulled down rather than left floating, so an unconnected pin reads a
    // steady LOW instead of picking up noise and reporting a touch that never
    // happened. The module drives this line hard enough to win against the
    // internal resistor either way round.
    pinMode(PIN_FP_TOUCH, INPUT_PULLDOWN);
    delay(10);

    const bool idleLevel = digitalRead(PIN_FP_TOUCH);
    Serial.print(F("  idle level: "));
    Serial.println(idleLevel == HIGH ? F("HIGH") : F("LOW"));
    Serial.println(F("  now touch the metal ring and hold it. 10 seconds."));
    showAll("Touch test", "", "Touch the metal", "ring and hold it");

    const uint32_t started = millis();
    bool changed = false;
    while (millis() - started < 10000) {
        if (digitalRead(PIN_FP_TOUCH) != idleLevel) {
            changed = true;
            break;
        }
        delay(20);
    }

    if (!changed) {
        Serial.println(F("  the pin never moved."));
        Serial.println(F("  1. blue on GPIO 4"));
        Serial.println(F("  2. green on 3V3. Without it the ring has no supply"));
        Serial.println(F("     and the touch output never does anything."));
        Serial.println(F("  3. some AS608 modules ship without the touch ring at"));
        Serial.println(F("     all, in which case leave both wires off and set"));
        Serial.println(F("     TOUCH_WIRED to false. Nothing else needs it."));
        showAll("No touch detected", "", "Check blue on GPIO4", "and green on 3V3");
        delay(2500);
        idle();
        return;
    }

    touchActiveHigh = (idleLevel == LOW);
    Serial.print(F("  it works, and it goes "));
    Serial.print(touchActiveHigh ? F("HIGH") : F("LOW"));
    Serial.println(F(" on touch."));
    Serial.print(F("  set  touchActiveHigh = "));
    Serial.println(touchActiveHigh ? F("true;") : F("false;"));
    Serial.println(F("  and TOUCH_WIRED = true; to have the sensor sleep until"));
    Serial.println(F("  somebody actually reaches for it."));

    showAll("The touch wire works", "",
            touchActiveHigh ? "It goes HIGH" : "It goes LOW",
            "when touched");

    Serial.println(F("  lift your finger."));
    while (digitalRead(PIN_FP_TOUCH) != idleLevel) {
        delay(20);
    }
    Serial.println(F("  released."));
    delay(1200);
    idle();
}

void doEmpty() {
    Serial.println(F("This erases every stored finger on this sensor."));
    Serial.println(F("Everyone bound to this reader would have to enroll again."));
    Serial.println(F("Type YES within 10 seconds to go ahead."));
    showAll("Erase everything?", "", "Everyone would have", "to enroll again");

    const uint32_t started = millis();
    while (!Serial.available() && millis() - started < 10000) {
        delay(20);
    }
    String answer = Serial.readStringUntil('\n');
    answer.trim();

    if (answer != "YES") {
        Serial.println(F("  left alone."));
        showAll("Left alone", "", "Nothing was erased", "");
        delay(1500);
        idle();
        return;
    }

    if (finger.emptyDatabase() == FINGERPRINT_OK) {
        Serial.println(F("  library emptied."));
        showAll("All erased", "", "No fingers stored", "");
    } else {
        Serial.println(F("  failed."));
        showAll("Erase failed", "", "Please try again", "");
    }

    delay(1800);
    idle();
}

void setup() {
    Serial.begin(115200);
    delay(300);

    if (TOUCH_WIRED) {
        pinMode(PIN_FP_TOUCH, INPUT_PULLDOWN);
    }

    Serial.println();
    Serial.println(F("AS608 sensor and 20x4 display test."));

    // The display comes up first so it can report the sensor's own state.
    startLcd();
    showAll("SIGURADO", "", "Starting up", "");

    online = connect();
    if (online) {
        help();
        delay(1500);
        idle();
    }
}

void loop() {
    if (!online) {
        delay(3000);
        Serial.println(F("retrying the handshake..."));
        online = connect();
        if (online) {
            help();
            delay(1500);
            idle();
        }
        return;
    }

    if (!Serial.available()) {
        return;
    }

    const char c = Serial.read();
    switch (c) {
        case 's':
            doSearch();
            break;
        case 'e':
            doEnroll();
            break;
        case 'd':
            doDelete();
            break;
        case 'c': {
            finger.getTemplateCount();
            Serial.print(F("templates stored: "));
            Serial.println(finger.templateCount);
            char count[LINE_SIZE];
            snprintf(count, sizeof(count), "%u finger(s) stored", finger.templateCount);
            showAll("Stored fingers", "", count, "");
            delay(1800);
            idle();
            break;
        }
        case 'l':
            doList();
            break;
        case 't':
            doTouch();
            break;
        case 'x':
            doEmpty();
            break;
        case '\n':
        case '\r':
            break;
        default:
            help();
            break;
    }
}
