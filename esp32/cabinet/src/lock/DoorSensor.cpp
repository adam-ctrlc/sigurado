#include "DoorSensor.h"

void DoorSensor::begin() {
    pinMode(pin_, INPUT_PULLUP);
    closed_ = rawClosed();
    lastRaw_ = closed_;
    lastRawChangeAt_ = millis();
}

// COM to GND, so an actuated lever shorts the pin. Which terminal that is
// depends on the wiring, which is what closedWhenLow_ records.
bool DoorSensor::rawClosed() const {
    const bool low = digitalRead(pin_) == LOW;
    return closedWhenLow_ ? low : !low;
}

bool DoorSensor::isClosed() const {
    return closed_;
}

uint32_t DoorSensor::openForMs() const {
    if (openedAt_ == 0) return 0;
    return millis() - openedAt_;
}

uint32_t DoorSensor::secondsToAlarm() const {
    const uint32_t open = openForMs();
    if (open >= alarmMs_) return 0;
    return (alarmMs_ - open) / 1000;
}

DoorEvent DoorSensor::update() {
    const uint32_t now = millis();
    const bool raw = rawClosed();

    // Every bounce restarts the clock, so the state only moves once the contact
    // has held still. Without this a slammed door reads closed, open, closed
    // within a few milliseconds and the open timer keeps resetting.
    if (raw != lastRaw_) {
        lastRaw_ = raw;
        lastRawChangeAt_ = now;
    }
    const bool settled = (now - lastRawChangeAt_) >= debounceMs_;

    if (settled && raw != closed_) {
        closed_ = raw;
        if (closed_) {
            openedAt_ = 0;
            alarmLatched_ = false;
            return DoorEvent::JustClosed;
        }
        openedAt_ = now;
        alarmLatched_ = false;
    }

    if (closed_) return DoorEvent::NoChange;

    const uint32_t open = openForMs();
    if (open > alarmMs_) return DoorEvent::Alarm;
    if (open > warnMs_) return DoorEvent::Warning;
    return DoorEvent::NoChange;
}
