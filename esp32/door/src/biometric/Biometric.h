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

inline bool Biometric::begin(uint8_t rxPin, uint8_t txPin, uint32_t baud) {
    port_.begin(baud, SERIAL_8N1, rxPin, txPin);
    delay(100);
    return finger_.verifyPassword();
}

inline FingerScan Biometric::poll() {
    const uint8_t img = finger_.getImage();
    if (img == FINGERPRINT_NOFINGER) return FingerScan::None;
    if (img != FINGERPRINT_OK) return FingerScan::Error;

    if (finger_.image2Tz() != FINGERPRINT_OK) return FingerScan::Error;

    if (finger_.fingerFastSearch() != FINGERPRINT_OK) return FingerScan::NoMatch;

    lastSlot_ = finger_.fingerID;
    lastConfidence_ = finger_.confidence;
    return FingerScan::Matched;
}

inline uint16_t Biometric::templateCount() {
    finger_.getTemplateCount();
    return finger_.templateCount;
}

inline int Biometric::nextFreeSlot(uint16_t maxSlot) {
    for (uint16_t slot = 1; slot <= maxSlot; slot++) {
        if (finger_.loadModel(slot) != FINGERPRINT_OK) return slot;
    }
    return -1;
}

inline bool Biometric::erase(uint16_t slot) {
    return finger_.deleteModel(slot) == FINGERPRINT_OK;
}

inline bool Biometric::waitRemoved(uint32_t timeoutMs) {
    const uint32_t started = millis();
    while (millis() - started < timeoutMs) {
        if (finger_.getImage() == FINGERPRINT_NOFINGER) return true;
        delay(60);
    }
    return false;
}

inline bool Biometric::captureInto(uint8_t buffer, uint32_t timeoutMs) {
    const uint32_t started = millis();
    while (millis() - started < timeoutMs) {
        if (finger_.getImage() == FINGERPRINT_OK) {
            return finger_.image2Tz(buffer) == FINGERPRINT_OK;
        }
        delay(60);
    }
    return false;
}

inline bool Biometric::enroll(uint16_t slot, void (*onStep)(const char*)) {
    if (onStep) onStep("Place finger");
    if (!captureInto(1, 15000)) return false;

    if (onStep) onStep("Remove finger");
    waitRemoved(8000);

    if (onStep) onStep("Place again");
    if (!captureInto(2, 15000)) return false;

    if (finger_.createModel() != FINGERPRINT_OK) return false;
    return finger_.storeModel(slot) == FINGERPRINT_OK;
}
