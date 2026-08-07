#include "ApiClient.h"

namespace {

// Copies a JSON string field into a fixed buffer, always NUL terminated.
void copyField(JsonVariantConst v, char* out, size_t n) {
    const char* s = v.as<const char*>();
    if (!s) {
        out[0] = '\0';
        return;
    }
    strncpy(out, s, n - 1);
    out[n - 1] = '\0';
}

}  // namespace

void ApiClient::tokenFor(uint16_t slot, char* out, size_t n) const {
    snprintf(out, n, "%s:%u", tag_, static_cast<unsigned>(slot));
}

ScanResult ApiClient::scan(const char* fingerToken) {
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

EnrollCodeResult ApiClient::requestEnrollCode() {
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

BindResult ApiClient::bind(const char* fingerToken) {
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

HeartbeatResult ApiClient::heartbeat() {
    return beat("{}");
}

HeartbeatResult ApiClient::heartbeat(bool doorClosed) {
    return beat(doorClosed ? "{\"door_closed\":true}"
                           : "{\"door_closed\":false}");
}

HeartbeatResult ApiClient::beat(const char* body) {
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

int ApiClient::pendingSms(SmsJob* out, size_t max) {
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

bool ApiClient::reportSms(const char* id, bool sent, const char* error) {
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
