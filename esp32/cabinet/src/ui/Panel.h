#pragma once

#include <Arduino.h>
#include <LiquidCrystal_I2C.h>
#include <Wire.h>
#include <stdarg.h>

// The front panel: 16x2 I2C LCD, status LED, enroll button and an optional
// buzzer. Pass buzzerPin = 255 on a node with no buzzer.
//
// MEMORY: takes const char* and formats into a fixed internal buffer. The old
// String + substring() version allocated twice per refresh, which at one
// refresh a second is millions of allocations a day and a fragmented heap.
class Panel {
  public:
    static const uint8_t NO_BUZZER = 255;

    Panel(uint8_t lcdAddress, uint8_t ledPin, uint8_t buttonPin,
          uint8_t buzzerPin = NO_BUZZER)
        : lcd_(lcdAddress, 16, 2), ledPin_(ledPin), buttonPin_(buttonPin),
          buzzerPin_(buzzerPin) {}

    void begin(uint8_t sdaPin, uint8_t sclPin);

    void show(const char* line1, const char* line2 = "");
    void showf(const char* line1, const char* fmt, ...);

    // Skips the write if the text has not changed, so a per-second idle
    // refresh costs nothing.
    void showIfChanged(const char* line1, const char* line2);

    void led(bool on);
    void blink(uint8_t times, uint16_t onMs = 120);
    void chirp(uint16_t ms = 90);
    void buzzer(bool on);

    // Debounced, and waits for release so one press is one event.
    bool buttonPressed();

  private:
    LiquidCrystal_I2C lcd_;
    uint8_t ledPin_;
    uint8_t buttonPin_;
    uint8_t buzzerPin_;
    char shown1_[17] = {0};
    char shown2_[17] = {0};

    void writeLine(uint8_t row, const char* text);
};

inline void Panel::begin(uint8_t sdaPin, uint8_t sclPin) {
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
inline void Panel::writeLine(uint8_t row, const char* text) {
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

inline void Panel::show(const char* line1, const char* line2) {
    writeLine(0, line1);
    writeLine(1, line2);
    strncpy(shown1_, line1, sizeof(shown1_) - 1);
    shown1_[sizeof(shown1_) - 1] = '\0';
    strncpy(shown2_, line2, sizeof(shown2_) - 1);
    shown2_[sizeof(shown2_) - 1] = '\0';
}

inline void Panel::showIfChanged(const char* line1, const char* line2) {
    if (strncmp(shown1_, line1, 16) == 0 && strncmp(shown2_, line2, 16) == 0) {
        return;
    }
    show(line1, line2);
}

inline void Panel::showf(const char* line1, const char* fmt, ...) {
    char buf[17];
    va_list args;
    va_start(args, fmt);
    vsnprintf(buf, sizeof(buf), fmt, args);
    va_end(args);
    show(line1, buf);
}

inline void Panel::led(bool on) {
    digitalWrite(ledPin_, on ? HIGH : LOW);
}

inline void Panel::blink(uint8_t times, uint16_t onMs) {
    for (uint8_t i = 0; i < times; i++) {
        led(true);
        delay(onMs);
        led(false);
        delay(onMs);
    }
}

inline void Panel::chirp(uint16_t ms) {
    if (buzzerPin_ == NO_BUZZER) return;
    digitalWrite(buzzerPin_, HIGH);
    delay(ms);
    digitalWrite(buzzerPin_, LOW);
}

inline void Panel::buzzer(bool on) {
    if (buzzerPin_ == NO_BUZZER) return;
    digitalWrite(buzzerPin_, on ? HIGH : LOW);
}

inline bool Panel::buttonPressed() {
    if (digitalRead(buttonPin_) != LOW) return false;
    delay(40);
    if (digitalRead(buttonPin_) != LOW) return false;
    while (digitalRead(buttonPin_) == LOW) delay(10);
    return true;
}
