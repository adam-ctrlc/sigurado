#include "Biometric.h"

bool Biometric::begin(uint8_t rxPin, uint8_t txPin, uint32_t baud) {
    port_.begin(baud, SERIAL_8N1, rxPin, txPin);
    delay(100);
    return finger_.verifyPassword();
}

FingerScan Biometric::poll() {
    const uint8_t img = finger_.getImage();
    if (img == FINGERPRINT_NOFINGER) return FingerScan::None;
    if (img != FINGERPRINT_OK) return FingerScan::Error;

    if (finger_.image2Tz() != FINGERPRINT_OK) return FingerScan::Error;

    if (finger_.fingerFastSearch() != FINGERPRINT_OK) return FingerScan::NoMatch;

    lastSlot_ = finger_.fingerID;
    lastConfidence_ = finger_.confidence;
    return FingerScan::Matched;
}

uint16_t Biometric::templateCount() {
    finger_.getTemplateCount();
    return finger_.templateCount;
}

int Biometric::nextFreeSlot(uint16_t maxSlot) {
    for (uint16_t slot = 1; slot <= maxSlot; slot++) {
        if (finger_.loadModel(slot) != FINGERPRINT_OK) return slot;
    }
    return -1;
}

bool Biometric::erase(uint16_t slot) {
    return finger_.deleteModel(slot) == FINGERPRINT_OK;
}

bool Biometric::waitRemoved(uint32_t timeoutMs) {
    const uint32_t started = millis();
    while (millis() - started < timeoutMs) {
        if (finger_.getImage() == FINGERPRINT_NOFINGER) return true;
        delay(60);
    }
    return false;
}

bool Biometric::captureInto(uint8_t buffer, uint32_t timeoutMs) {
    const uint32_t started = millis();
    while (millis() - started < timeoutMs) {
        if (finger_.getImage() == FINGERPRINT_OK) {
            return finger_.image2Tz(buffer) == FINGERPRINT_OK;
        }
        delay(60);
    }
    return false;
}

bool Biometric::enroll(uint16_t slot, void (*onStep)(const char*)) {
    if (onStep) onStep("Place finger");
    if (!captureInto(1, 15000)) return false;

    if (onStep) onStep("Remove finger");
    waitRemoved(8000);

    if (onStep) onStep("Place again");
    if (!captureInto(2, 15000)) return false;

    if (finger_.createModel() != FINGERPRINT_OK) return false;
    return finger_.storeModel(slot) == FINGERPRINT_OK;
}
