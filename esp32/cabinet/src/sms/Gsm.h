#pragma once

#include <Arduino.h>

// SIM800L on its own hardware UART.
//
// The server has no modem, so it only ever queues a text. This node is the one
// that actually sends it, which is why the cabinet node is the one marked
// sms_capable in the device list.
//
// POWER: the SIM800L draws up to 2A in bursts while it talks to the tower and
// browns out on the ESP32 3V3 regulator. Give it its own 4V supply and a fat
// capacitor across the module, or it will reset mid-message and look like a
// software fault.
//
// TIMING: sending is slow, a few seconds per message, and blocking. The sketch
// drains the queue only while nobody is at the reader.
class Gsm {
  public:
    Gsm(HardwareSerial& port, uint8_t rxPin, uint8_t txPin, uint8_t resetPin)
        : port_(port), rxPin_(rxPin), txPin_(txPin), resetPin_(resetPin) {}

    // Opens the port and puts the modem in text mode. False means it never
    // answered, which is nearly always power rather than wiring.
    bool begin(uint32_t baud = 9600);

    bool ready() const { return ready_; }

    // Asks the modem whether it is on a network yet. Sending before this is
    // true fails, so the sketch waits.
    bool registered();

    // Signal strength, 0 to 31, or -1 if the modem did not answer.
    int signal();

    // Sends one text. `error` receives a short reason on failure, for the
    // audit trail rather than for the person.
    bool send(const char* number, const char* body, char* error, size_t errorN);

    // Pulls the reset line low briefly. Worth trying once before giving up.
    void hardReset();

  private:
    // Writes a command and waits for `expect` in the reply.
    bool command(const char* cmd, const char* expect, uint32_t timeoutMs,
                 char* reply = nullptr, size_t replyN = 0);
    void drain();

    HardwareSerial& port_;
    uint8_t rxPin_;
    uint8_t txPin_;
    uint8_t resetPin_;
    bool ready_ = false;
};
