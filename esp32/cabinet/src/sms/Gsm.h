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

constexpr uint32_t SHORT_WAIT_MS = 2000;
constexpr uint32_t SEND_WAIT_MS = 20000;   // a tower can take its time
constexpr char CTRL_Z = 26;

inline void Gsm::drain() {
    while (port_.available()) port_.read();
}

inline bool Gsm::command(const char* cmd, const char* expect, uint32_t timeoutMs,
                  char* reply, size_t replyN) {
    drain();
    port_.print(cmd);
    port_.print("\r\n");

    // Scanned as it arrives rather than collected, so a chatty modem cannot
    // outgrow a buffer.
    char window[64] = {0};
    size_t len = 0;
    const size_t expectLen = strlen(expect);
    const uint32_t started = millis();

    while (millis() - started < timeoutMs) {
        while (port_.available()) {
            const char c = (char)port_.read();
            if (len + 1 < sizeof(window)) {
                window[len++] = c;
                window[len] = '\0';
            } else {
                // Slide, keeping the tail long enough to still match.
                memmove(window, window + 1, len - 1);
                window[len - 1] = c;
            }
            if (reply != nullptr && replyN > 1) {
                strncpy(reply, window, replyN - 1);
                reply[replyN - 1] = '\0';
            }
            if (len >= expectLen && strstr(window, expect) != nullptr) {
                return true;
            }
            if (strstr(window, "ERROR") != nullptr) return false;
        }
        delay(5);
    }
    return false;
}

inline void Gsm::hardReset() {
    if (resetPin_ == 255) return;
    pinMode(resetPin_, OUTPUT);
    digitalWrite(resetPin_, LOW);
    delay(120);
    digitalWrite(resetPin_, HIGH);
    delay(3000);
}

inline bool Gsm::begin(uint32_t baud) {
    port_.begin(baud, SERIAL_8N1, rxPin_, txPin_);
    if (resetPin_ != 255) {
        pinMode(resetPin_, OUTPUT);
        digitalWrite(resetPin_, HIGH);
    }
    delay(1500);

    // Two tries: the module is often still booting when the ESP32 is ready.
    bool alive = command("AT", "OK", SHORT_WAIT_MS);
    if (!alive) alive = command("AT", "OK", SHORT_WAIT_MS);
    if (!alive) return false;

    command("ATE0", "OK", SHORT_WAIT_MS);          // no echo, easier to parse
    command("AT+CMGF=1", "OK", SHORT_WAIT_MS);     // text mode, not PDU
    command("AT+CSCS=\"GSM\"", "OK", SHORT_WAIT_MS);
    command("AT+CNMI=0,0,0,0,0", "OK", SHORT_WAIT_MS);  // do not push inbound

    ready_ = true;
    return true;
}

inline bool Gsm::registered() {
    char reply[64] = {0};
    if (!command("AT+CREG?", "+CREG:", SHORT_WAIT_MS, reply, sizeof(reply))) {
        return false;
    }
    // "+CREG: 0,1" is home, "0,5" is roaming. Anything else is not usable yet.
    const char* p = strstr(reply, "+CREG:");
    if (p == nullptr) return false;
    const char* comma = strchr(p, ',');
    if (comma == nullptr) return false;
    const char stat = comma[1];
    return stat == '1' || stat == '5';
}

inline int Gsm::signal() {
    char reply[64] = {0};
    if (!command("AT+CSQ", "+CSQ:", SHORT_WAIT_MS, reply, sizeof(reply))) {
        return -1;
    }
    const char* p = strstr(reply, "+CSQ:");
    if (p == nullptr) return -1;
    const int rssi = atoi(p + 5);
    return (rssi < 0 || rssi > 31) ? -1 : rssi;
}

inline bool Gsm::send(const char* number, const char* body, char* error,
               size_t errorN) {
    const auto fail = [&](const char* why) {
        if (error != nullptr && errorN > 0) {
            strncpy(error, why, errorN - 1);
            error[errorN - 1] = '\0';
        }
        return false;
    };

    if (!ready_) return fail("modem not ready");
    if (!registered()) return fail("no network");

    char cmd[40];
    snprintf(cmd, sizeof(cmd), "AT+CMGS=\"%s\"", number);
    if (!command(cmd, ">", SHORT_WAIT_MS * 2)) return fail("no prompt");

    port_.print(body);
    port_.write(CTRL_Z);

    // +CMGS on its own means the tower accepted it. Anything else is a failure
    // worth reporting, so the server can retry or give up.
    if (!command("", "+CMGS:", SEND_WAIT_MS)) return fail("send timed out");

    if (error != nullptr && errorN > 0) error[0] = '\0';
    return true;
}
