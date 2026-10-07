// TEST 2 of 12: the I2C bus.
//
// Run this before the LCD test. It tells you the address the LCD backpack
// actually answers on, which is the single most common reason a display stays
// blank: the sketch says 0x27 and the board shipped as 0x3F.
//
// Wiring, LCD backpack to ESP32:
//   VCC to VIN (5V from USB). The HD44780 will not light properly on 3.3V.
//   GND to GND
//   SDA to GPIO 21
//   SCL to GPIO 22
//
// The PCF8574 backpack is 5V, but its SDA and SCL only ever get pulled down, so
// the 3.3V ESP32 pins are safe on a short bench lead. Keep it under 20 cm.
//
// Expect: at least one address found. Put that number in Config.h as
// LCD_ADDRESS on both the door and the cabinet node.

#include <Wire.h>

static const uint8_t PIN_SDA = 21;
static const uint8_t PIN_SCL = 22;

static const uint32_t RESCAN_MS = 5000;

uint32_t lastScan = 0;

const char* guessDevice(uint8_t address) {
    switch (address) {
        case 0x20:
        case 0x21:
        case 0x22:
        case 0x23:
        case 0x24:
        case 0x25:
        case 0x26:
        case 0x27:
            return "PCF8574 (LCD backpack, most likely)";
        case 0x38:
        case 0x39:
        case 0x3A:
        case 0x3B:
        case 0x3C:
        case 0x3D:
        case 0x3E:
        case 0x3F:
            return "PCF8574A (LCD backpack) or SSD1306 OLED";
        case 0x68:
            return "DS3231/DS1307 clock or MPU6050";
        case 0x76:
        case 0x77:
            return "BME280/BMP280";
        default:
            return "unknown";
    }
}

void scan() {
    Serial.println();
    Serial.println(F("=== scanning 0x01 to 0x7E ==="));

    uint8_t found = 0;
    for (uint8_t address = 1; address < 127; address++) {
        Wire.beginTransmission(address);
        const uint8_t err = Wire.endTransmission();

        if (err == 0) {
            found++;
            Serial.print(F("  found 0x"));
            if (address < 16) {
                Serial.print('0');
            }
            Serial.print(address, HEX);
            Serial.print(F("  "));
            Serial.println(guessDevice(address));
        } else if (err == 4) {
            Serial.print(F("  bus error at 0x"));
            Serial.println(address, HEX);
        }
    }

    if (found == 0) {
        Serial.println(F("  nothing answered."));
        Serial.println();
        Serial.println(F("Check, in this order:"));
        Serial.println(F("  1. VCC on VIN, not on 3V3"));
        Serial.println(F("  2. GND shared with the ESP32"));
        Serial.println(F("  3. SDA on 21 and SCL on 22, not swapped"));
        Serial.println(F("  4. the backpack soldered to the display pins"));
    } else {
        Serial.print(F("  "));
        Serial.print(found);
        Serial.println(F(" device(s). Use the address above as LCD_ADDRESS."));
    }
}

void setup() {
    Serial.begin(115200);
    delay(300);

    Wire.begin(PIN_SDA, PIN_SCL);
    // 100 kHz. The cheap backpacks are unreliable at 400 kHz on long leads.
    Wire.setClock(100000);

    Serial.println();
    Serial.println(F("I2C scanner. Rescans every 5 seconds, so you can plug and"));
    Serial.println(F("unplug the display and watch it appear."));
    scan();
    lastScan = millis();
}

void loop() {
    if (millis() - lastScan >= RESCAN_MS) {
        lastScan = millis();
        scan();
    }
}
