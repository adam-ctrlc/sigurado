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
