#pragma once

#include <Arduino.h>

// Fail-secure 12V solenoid driven through a relay's dry contacts.
//
// POWER: the coil runs from its own 12V supply, never from the ESP32 rails.
// Fit a 1N4007 flyback diode across the coil (band to +12V) or the collapsing
// field will pit the relay contacts and brown out the board. Share only GND.
//
// The coil is energised for release() and then de-energised, so a short-duty
// solenoid never overheats. Re-latching is mechanical: the ramped tab on the
// door cams the spring bolt up as it swings shut, so closing needs no power.
class Solenoid {
  public:
    Solenoid(uint8_t pin, bool activeLow, uint32_t holdMs)
        : pin_(pin), activeLow_(activeLow), holdMs_(holdMs) {}

    void begin();
    void release();        // unlock for holdMs, then re-lock
    void lock();

  private:
    uint8_t pin_;
    bool activeLow_;
    uint32_t holdMs_;
};
