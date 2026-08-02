#pragma once

#include <Arduino.h>

// Lever limit switch on the carcass, pressed by a striker pad on the door.
//
// Without this the bolt can fire into thin air on an open door while the audit
// trail claims "secured". It also closes the loophole where someone scans in,
// leaves the door open and walks away.
//
// A limit switch is a mechanical contact, so it bounces on every actuation and
// chatters when the door is knocked. The state here is debounced, which a magnet
// and reed pair could get away without but a lever cannot.
enum class DoorEvent {
    NoChange,
    JustClosed,
    Warning,     // open past the warning threshold
    Alarm        // open past the alarm threshold
};

class DoorSensor {
  public:
    // `closedWhenLow` follows the wiring: COM to GND with the normally-open
    // terminal on the pin reads LOW when the door presses the lever. Wire the
    // normally-closed terminal instead and set this false.
    DoorSensor(uint8_t pin, uint32_t warnMs, uint32_t alarmMs,
               bool closedWhenLow = true, uint32_t debounceMs = 20)
        : pin_(pin), warnMs_(warnMs), alarmMs_(alarmMs),
          closedWhenLow_(closedWhenLow), debounceMs_(debounceMs) {}

    void begin();

    // The settled state, not the pin. Safe to call as often as you like.
    bool isClosed() const;

    // Call every loop. Returns what changed so the sketch can react.
    DoorEvent update();

    uint32_t openForMs() const;
    uint32_t secondsToAlarm() const;

  private:
    // The pin as it reads right now, contact bounce and all.
    bool rawClosed() const;

    uint8_t pin_;
    uint32_t warnMs_;
    uint32_t alarmMs_;
    bool closedWhenLow_;
    uint32_t debounceMs_;
    uint32_t openedAt_ = 0;
    bool closed_ = true;        // the settled state
    bool lastRaw_ = true;       // last raw read, for timing the settle
    uint32_t lastRawChangeAt_ = 0;
    bool alarmLatched_ = false;
};

inline void DoorSensor::begin() {
    pinMode(pin_, INPUT_PULLUP);
    closed_ = rawClosed();
    lastRaw_ = closed_;
    lastRawChangeAt_ = millis();
}

// COM to GND, so an actuated lever shorts the pin. Which terminal that is
// depends on the wiring, which is what closedWhenLow_ records.
inline bool DoorSensor::rawClosed() const {
    const bool low = digitalRead(pin_) == LOW;
    return closedWhenLow_ ? low : !low;
}

inline bool DoorSensor::isClosed() const {
    return closed_;
}

inline uint32_t DoorSensor::openForMs() const {
    if (openedAt_ == 0) return 0;
    return millis() - openedAt_;
}

inline uint32_t DoorSensor::secondsToAlarm() const {
    const uint32_t open = openForMs();
    if (open >= alarmMs_) return 0;
    return (alarmMs_ - open) / 1000;
}

inline DoorEvent DoorSensor::update() {
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
