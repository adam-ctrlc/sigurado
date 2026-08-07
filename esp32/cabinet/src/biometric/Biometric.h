#pragma once

#include <Adafruit_Fingerprint.h>
#include <Arduino.h>

enum class FingerScan {
    None,       // nothing on the platen
    Matched,    // matched a stored template, see lastSlot()
    NoMatch,    // read a finger but no stored template matched
    Error
};

// AS608 / R307 on a hardware UART.
//
// Matching happens on the sensor against templates in ITS OWN flash, so slot
// numbers are local to this reader. That is why tokens are namespaced per
// device and why a user enrolls at the door and at the cabinet separately.
class Biometric {
  public:
    explicit Biometric(HardwareSerial& port) : port_(port), finger_(&port) {}

    bool begin(uint8_t rxPin, uint8_t txPin, uint32_t baud = 57600);

    FingerScan poll();
    uint16_t lastSlot() const { return lastSlot_; }
    uint16_t lastConfidence() const { return lastConfidence_; }

    uint16_t templateCount();
    int nextFreeSlot(uint16_t maxSlot = 127);
    bool erase(uint16_t slot);
    bool waitRemoved(uint32_t timeoutMs);

    // Two-capture enrollment into the given slot. onStep, if supplied, is
    // called with a short prompt for the LCD.
    bool enroll(uint16_t slot, void (*onStep)(const char*) = nullptr);

  private:
    HardwareSerial& port_;
    Adafruit_Fingerprint finger_;
    uint16_t lastSlot_ = 0;
    uint16_t lastConfidence_ = 0;

    bool captureInto(uint8_t buffer, uint32_t timeoutMs);
};
