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

namespace api_detail {

// Copies a JSON string field into a fixed buffer, always NUL terminated.
inline void copyField(JsonVariantConst v, char* out, size_t n) {
    const char* s = v.as<const char*>();
    if (!s) {
        out[0] = '\0';
        return;
    }
    strncpy(out, s, n - 1);
    out[n - 1] = '\0';
}

}  // namespace api_detail

using api_detail::copyField;

inline void ApiClient::tokenFor(uint16_t slot, char* out, size_t n) const {
    snprintf(out, n, "%s:%u", tag_, static_cast<unsigned>(slot));
}

inline ScanResult ApiClient::scan(const char* fingerToken) {
    ScanResult out;

    char body[96];
    snprintf(body, sizeof(body), "{\"finger_token\":\"%s\"}", fingerToken);

    JsonDocument filter;
    filter["action"] = true;
    filter["reason"] = true;
    filter["enroll_hint"] = true;
    filter["session_expires_at"] = true;
    filter["checkout_code"] = true;
    filter["user"]["display_name"] = true;

    JsonDocument doc;
    out.httpCode = net_.postParse("/device/scan", body, doc, filter);
    if (out.httpCode != 200) return out;

    const char* action = doc["action"].as<const char*>();
    out.ok = true;
    out.unlock = action && strcmp(action, "unlock") == 0;
    out.enrollHint = doc["enroll_hint"] | false;
    copyField(doc["reason"], out.reason, sizeof(out.reason));
    copyField(doc["session_expires_at"], out.sessionExpiresAt,
              sizeof(out.sessionExpiresAt));
    copyField(doc["user"]["display_name"], out.userName, sizeof(out.userName));
    copyField(doc["checkout_code"], out.checkoutCode, sizeof(out.checkoutCode));
    return out;
}

inline EnrollCodeResult ApiClient::requestEnrollCode() {
    EnrollCodeResult out;

    JsonDocument filter;
    filter["code"] = true;
    filter["code_expires_at"] = true;

    JsonDocument doc;
    out.httpCode = net_.postParse("/device/enroll/request-code", "{}", doc, filter);
    if (out.httpCode != 200) return out;

    copyField(doc["code"], out.code, sizeof(out.code));
    copyField(doc["code_expires_at"], out.expiresAt, sizeof(out.expiresAt));
    out.ok = out.code[0] != '\0';
    return out;
}

inline BindResult ApiClient::bind(const char* fingerToken) {
    BindResult out;

    char body[96];
    snprintf(body, sizeof(body), "{\"finger_token\":\"%s\"}", fingerToken);

    JsonDocument filter;
    filter["bound"] = true;
    filter["message"] = true;
    filter["user"]["display_name"] = true;

    JsonDocument doc;
    out.httpCode = net_.postParse("/device/enroll/bind", body, doc, filter);
    if (out.httpCode != 200) return out;

    out.ok = true;
    out.bound = doc["bound"] | false;
    copyField(doc["message"], out.message, sizeof(out.message));
    copyField(doc["user"]["display_name"], out.userName, sizeof(out.userName));
    return out;
}

inline HeartbeatResult ApiClient::heartbeat() {
    return beat("{}");
}

inline HeartbeatResult ApiClient::heartbeat(bool doorClosed) {
    return beat(doorClosed ? "{\"door_closed\":true}"
                           : "{\"door_closed\":false}");
}

inline HeartbeatResult ApiClient::beat(const char* body) {
    HeartbeatResult out;

    JsonDocument filter;
    filter["sms_capable"] = true;
    filter["pending_sms"] = true;

    JsonDocument doc;
    if (net_.postParse("/device/heartbeat", body, doc, filter) != 200) {
        return out;
    }

    out.ok = true;
    out.smsCapable = doc["sms_capable"] | false;
    out.pendingSms = doc["pending_sms"] | 0u;
    return out;
}

inline int ApiClient::pendingSms(SmsJob* out, size_t max) {
    JsonDocument filter;
    JsonObject shape = filter.add<JsonObject>();
    shape["id"] = true;
    shape["phone_number"] = true;
    shape["body"] = true;
    shape["attempts"] = true;

    JsonDocument doc;
    const int code = net_.get("/device/sms/pending", doc, filter);
    if (code != 200) return code < 0 ? code : -code;

    size_t n = 0;
    for (JsonObjectConst job : doc.as<JsonArrayConst>()) {
        if (n >= max) break;
        copyField(job["id"], out[n].id, sizeof(out[n].id));
        copyField(job["phone_number"], out[n].number, sizeof(out[n].number));
        copyField(job["body"], out[n].body, sizeof(out[n].body));
        out[n].attempts = job["attempts"] | 0;
        if (out[n].id[0] != '\0' && out[n].number[0] != '\0') n++;
    }
    return (int)n;
}

inline bool ApiClient::reportSms(const char* id, bool sent, const char* error) {
    char path[80];
    snprintf(path, sizeof(path), "/device/sms/%s/result", id);

    char body[160];
    if (sent) {
        snprintf(body, sizeof(body), "{\"sent\":true}");
    } else {
        // Quoted plainly: the reasons this reports are short AT replies with no
        // characters JSON would object to.
        snprintf(body, sizeof(body), "{\"sent\":false,\"error\":\"%s\"}",
                 error ? error : "modem refused");
    }
    return net_.post(path, body) == 200;
}
