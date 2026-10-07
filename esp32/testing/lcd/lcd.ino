// TEST 3 of 12: the 20x4 I2C display.
//
// Run i2c_scan first and put the address it reported in LCD_ADDRESS below.
//
// Wiring:
//   VCC to VIN (5V). The HD44780 will not light properly on 3.3V.
//   GND to GND
//   SDA to GPIO 21
//   SCL to GPIO 22
//
// The backpack is the same PCF8574 on a 16x2 and a 20x4, so the address tells
// you nothing about the size. If the text wraps into the wrong rows, or rows 3
// and 4 stay blank, the size is wrong rather than the address: press 'z'.
//
// Expect: four rows of text, then a screen that counts seconds. If the
// backlight is on but the rows are blank boxes or blank white, the address is
// right and the contrast is wrong: turn the small blue trimmer on the back of
// the backpack slowly with a screwdriver until the text appears.
//
// Monitor commands:
//   1  fill all four rows
//   2  walk the cursor through every cell, so a dead one shows up
//   3  the real messages the firmware shows, one every 2 seconds
//   4  a ruler, to count the columns you actually have
//   b  toggle the backlight
//   a  try the other common address and re-init
//   z  switch between 20x4 and 16x2

#include <LiquidCrystal_I2C.h>
#include <Wire.h>

static const uint8_t PIN_SDA = 21;
static const uint8_t PIN_SCL = 22;

static uint8_t lcdAddress = 0x27;

static uint8_t lcdCols = 20;
static uint8_t lcdRows = 4;

LiquidCrystal_I2C lcd(lcdAddress, lcdCols, lcdRows);

bool backlightOn = true;
uint32_t lastTick = 0;
uint32_t seconds = 0;
bool counting = true;

// The wording here is copied from the firmware on purpose. Reading it on the
// real glass is the only way to catch a message that overruns the row.
struct Message {
    const char* top;
    const char* bottom;
};

static const Message FIRMWARE_MESSAGES[] = {
    {"Ready", "Place your finger"},
    {"Welcome back", "Elena Reyes"},
    {"The door is open", "Please come in"},
    {"The cabinet is open", "Your code is 4821"},
    {"Not recognized", "Please try again"},
    {"Scan at the door", "first, then here"},
    {"Your enrollment code", "739104"},
    {"Your finger is saved", "You are all set"},
    {"The door is open", "Please close it"},
    {"Offline", "Retrying shortly"},
};
static const size_t MESSAGE_COUNT =
    sizeof(FIRMWARE_MESSAGES) / sizeof(FIRMWARE_MESSAGES[0]);

void clearRow(uint8_t row) {
    lcd.setCursor(0, row);
    for (uint8_t col = 0; col < lcdCols; col++) {
        lcd.print(' ');
    }
    lcd.setCursor(0, row);
}

void show(const char* top, const char* bottom) {
    lcd.clear();
    lcd.setCursor(0, 0);
    lcd.print(top);
    lcd.setCursor(0, 1);
    lcd.print(bottom);

    if (strlen(top) > lcdCols || strlen(bottom) > lcdCols) {
        Serial.print(F("  WARNING: longer than "));
        Serial.print(lcdCols);
        Serial.println(F(" characters, so it is cut off"));
    }
}

void startLcd() {
    lcd = LiquidCrystal_I2C(lcdAddress, lcdCols, lcdRows);
    lcd.init();
    lcd.backlight();
    backlightOn = true;

    Serial.print(F("initialised at 0x"));
    Serial.print(lcdAddress, HEX);
    Serial.print(F(" as "));
    Serial.print(lcdCols);
    Serial.print('x');
    Serial.println(lcdRows);

    fillRows();
}

void fillRows() {
    counting = false;
    lcd.clear();
    lcd.setCursor(0, 0);
    lcd.print("SIGURADO");
    lcd.setCursor(0, 1);
    lcd.print("display test");
    if (lcdRows > 2) {
        lcd.setCursor(0, 2);
        lcd.print("row three");
        lcd.setCursor(0, 3);
        lcd.print("row four");
    }
    Serial.print(F("  all "));
    Serial.print(lcdRows);
    Serial.println(F(" rows written. Every one should have text."));
    Serial.println(F("  If the last two are blank, press 'z'."));
}

// A 20 column ruler. Counting the digits on the glass is the quickest way to
// settle whether a module really is 20 wide.
void ruler() {
    counting = false;
    lcd.clear();
    lcd.setCursor(0, 0);
    lcd.print("12345678901234567890");
    lcd.setCursor(0, 1);
    lcd.print("....5....0....5....0");
    if (lcdRows > 2) {
        lcd.setCursor(0, 2);
        lcd.print("count the digits on");
        lcd.setCursor(0, 3);
        lcd.print("the top row");
    }
    Serial.println(F("  count the digits on the top row. 20 means 20 columns,"));
    Serial.println(F("  16 means this is a 16x2 and you should press 'z'."));
}

void walkCursor() {
    counting = false;
    lcd.clear();
    for (uint8_t row = 0; row < lcdRows; row++) {
        for (uint8_t col = 0; col < lcdCols; col++) {
            lcd.setCursor(col, row);
            lcd.print(char('0' + (col % 10)));
            delay(40);
        }
    }
    Serial.print(F("  all "));
    Serial.print((uint16_t)lcdCols * lcdRows);
    Serial.println(F(" cells written. Every one should be readable."));
}

void playMessages() {
    counting = false;
    for (size_t i = 0; i < MESSAGE_COUNT; i++) {
        Serial.print(F("  ["));
        Serial.print(FIRMWARE_MESSAGES[i].top);
        Serial.print(F(" / "));
        Serial.print(FIRMWARE_MESSAGES[i].bottom);
        Serial.println(F("]"));
        show(FIRMWARE_MESSAGES[i].top, FIRMWARE_MESSAGES[i].bottom);
        delay(2000);
    }
    Serial.println(F("  done. Back to the counter."));
    counting = true;
}

void help() {
    Serial.println();
    Serial.println(F("1 fill rows  2 cursor walk  3 firmware messages  4 ruler"));
    Serial.println(F("b backlight  a other address  z 20x4 / 16x2"));
}

void setup() {
    Serial.begin(115200);
    delay(300);

    Wire.begin(PIN_SDA, PIN_SCL);
    Wire.setClock(100000);

    Serial.println();
    Serial.println(F("LCD test."));
    startLcd();
    help();
}

void loop() {
    if (counting && millis() - lastTick >= 1000) {
        lastTick = millis();
        char line[24];
        snprintf(line, sizeof(line), "up %lus", (unsigned long)seconds++);
        clearRow(lcdRows > 2 ? 3 : 1);
        lcd.print(line);
    }

    if (!Serial.available()) {
        return;
    }

    const char c = Serial.read();
    switch (c) {
        case '1':
            fillRows();
            counting = true;
            break;
        case '2':
            walkCursor();
            break;
        case '3':
            playMessages();
            break;
        case '4':
            ruler();
            break;
        case 'b':
            backlightOn = !backlightOn;
            if (backlightOn) {
                lcd.backlight();
            } else {
                lcd.noBacklight();
            }
            Serial.print(F("  backlight "));
            Serial.println(backlightOn ? F("on") : F("off"));
            break;
        case 'a':
            lcdAddress = (lcdAddress == 0x27) ? 0x3F : 0x27;
            startLcd();
            break;
        case 'z':
            if (lcdCols == 20) {
                lcdCols = 16;
                lcdRows = 2;
            } else {
                lcdCols = 20;
                lcdRows = 4;
            }
            startLcd();
            break;
        case '\n':
        case '\r':
            break;
        default:
            help();
            break;
    }
}
