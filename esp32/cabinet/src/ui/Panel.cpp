#include "Panel.h"

#include <Wire.h>
#include <stdarg.h>

void Panel::begin(uint8_t sdaPin, uint8_t sclPin) {
    pinMode(ledPin_, OUTPUT);
    digitalWrite(ledPin_, LOW);
    pinMode(buttonPin_, INPUT_PULLUP);
    if (buzzerPin_ != NO_BUZZER) {
        pinMode(buzzerPin_, OUTPUT);
        digitalWrite(buzzerPin_, LOW);
    }

    Wire.begin(sdaPin, sclPin);
    lcd_.init();
    lcd_.backlight();
}

// Pads to a full 16 columns so we never need lcd_.clear(), which flickers.
void Panel::writeLine(uint8_t row, const char* text) {
    char padded[17];
    for (uint8_t i = 0; i < 16; i++) {
        padded[i] = text[i] ? text[i] : ' ';
        if (!text[i]) {
            for (uint8_t j = i; j < 16; j++) padded[j] = ' ';
            break;
        }
    }
    padded[16] = '\0';

    lcd_.setCursor(0, row);
    lcd_.print(padded);
}

void Panel::show(const char* line1, const char* line2) {
    writeLine(0, line1);
    writeLine(1, line2);
    strncpy(shown1_, line1, sizeof(shown1_) - 1);
    shown1_[sizeof(shown1_) - 1] = '\0';
    strncpy(shown2_, line2, sizeof(shown2_) - 1);
    shown2_[sizeof(shown2_) - 1] = '\0';
}

void Panel::showIfChanged(const char* line1, const char* line2) {
    if (strncmp(shown1_, line1, 16) == 0 && strncmp(shown2_, line2, 16) == 0) {
        return;
    }
    show(line1, line2);
}

void Panel::showf(const char* line1, const char* fmt, ...) {
    char buf[17];
    va_list args;
    va_start(args, fmt);
    vsnprintf(buf, sizeof(buf), fmt, args);
    va_end(args);
    show(line1, buf);
}

void Panel::led(bool on) {
    digitalWrite(ledPin_, on ? HIGH : LOW);
}

void Panel::blink(uint8_t times, uint16_t onMs) {
    for (uint8_t i = 0; i < times; i++) {
        led(true);
        delay(onMs);
        led(false);
        delay(onMs);
    }
}

void Panel::chirp(uint16_t ms) {
    if (buzzerPin_ == NO_BUZZER) return;
    digitalWrite(buzzerPin_, HIGH);
    delay(ms);
    digitalWrite(buzzerPin_, LOW);
}

void Panel::buzzer(bool on) {
    if (buzzerPin_ == NO_BUZZER) return;
    digitalWrite(buzzerPin_, on ? HIGH : LOW);
}

bool Panel::buttonPressed() {
    if (digitalRead(buttonPin_) != LOW) return false;
    delay(40);
    if (digitalRead(buttonPin_) != LOW) return false;
    while (digitalRead(buttonPin_) == LOW) delay(10);
    return true;
}
