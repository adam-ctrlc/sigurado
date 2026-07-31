#pragma once

#include <Arduino.h>
#include <LiquidCrystal_I2C.h>

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
