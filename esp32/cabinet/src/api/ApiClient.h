#pragma once

#include <Arduino.h>

#include "../net/Net.h"

// Fixed-size results: no heap, so these can be created in the hot loop forever
// without fragmenting anything.
struct ScanResult {
    bool ok = false;          // request completed and parsed
    int httpCode = 0;
    bool unlock = false;
    bool enrollHint = false;
    char reason[28] = {0};
    char userName[40] = {0};
    char sessionExpiresAt[36] = {0};
    // Only the cabinet gets one, and only when it actually unlocks. This is
    // what the person types on the website to record what they took, so it has
    // to reach the LCD and nowhere else.
    char checkoutCode[12] = {0};
};

struct EnrollCodeResult {
    bool ok = false;
    int httpCode = 0;
    char code[12] = {0};
    char expiresAt[36] = {0};
};

struct BindResult {
    bool ok = false;
    int httpCode = 0;
    bool bound = false;
    char message[64] = {0};
    char userName[40] = {0};
};

struct HeartbeatResult {
    bool ok = false;
    bool smsCapable = false;
    // How many texts are waiting. Zero means do not bother polling the queue.
    uint32_t pendingSms = 0;
};

// One queued text, as the modem needs it.
struct SmsJob {
    char id[40] = {0};
    char number[20] = {0};
    char body[168] = {0};
    int attempts = 0;
};

// Typed wrapper over the /api/device endpoints.
class ApiClient {
  public:
    ApiClient(Net& net, const char* deviceTag) : net_(net), tag_(deviceTag) {}

    // Namespaced token: matching is local to this sensor, so a slot number is
    // only meaningful together with the device tag.
    void tokenFor(uint16_t slot, char* out, size_t n) const;

    ScanResult scan(const char* fingerToken);
    EnrollCodeResult requestEnrollCode();
    BindResult bind(const char* fingerToken);

    // `doorClosed` is what stops the trail claiming "secured" while the door
    // stands open.
    HeartbeatResult heartbeat();
    HeartbeatResult heartbeat(bool doorClosed);

    // Pulls up to `max` queued texts. Returns how many landed in `out`, or a
    // negative HTTP/parse code on failure.
    int pendingSms(SmsJob* out, size_t max);

    // Tells the server what the modem did, so a message is retried or retired.
    bool reportSms(const char* id, bool sent, const char* error);

  private:
    HeartbeatResult beat(const char* body);

    Net& net_;
    const char* tag_;
};
