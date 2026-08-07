#include "Gsm.h"

namespace {
const uint32_t SHORT_WAIT_MS = 2000;
const uint32_t SEND_WAIT_MS = 20000;   // a tower can take its time
const char CTRL_Z = 26;
}  // namespace

void Gsm::drain() {
    while (port_.available()) port_.read();
}

bool Gsm::command(const char* cmd, const char* expect, uint32_t timeoutMs,
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

void Gsm::hardReset() {
    if (resetPin_ == 255) return;
    pinMode(resetPin_, OUTPUT);
    digitalWrite(resetPin_, LOW);
    delay(120);
    digitalWrite(resetPin_, HIGH);
    delay(3000);
}

bool Gsm::begin(uint32_t baud) {
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

bool Gsm::registered() {
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

int Gsm::signal() {
    char reply[64] = {0};
    if (!command("AT+CSQ", "+CSQ:", SHORT_WAIT_MS, reply, sizeof(reply))) {
        return -1;
    }
    const char* p = strstr(reply, "+CSQ:");
    if (p == nullptr) return -1;
    const int rssi = atoi(p + 5);
    return (rssi < 0 || rssi > 31) ? -1 : rssi;
}

bool Gsm::send(const char* number, const char* body, char* error,
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
