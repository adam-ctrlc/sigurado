#include "Net.h"

#include <HTTPClient.h>
#include <WiFi.h>
#include <time.h>

namespace {
// Rate limit reconnect storms: each WiFi.begin() churns the heap.
const uint32_t RECONNECT_MIN_GAP_MS = 5000;
}  // namespace

void Net::buildUrl(const char* path, char* out, size_t n) const {
    snprintf(out, n, "%s%s", apiBase_, path);
}

void Net::addHeaders(HTTPClient& http) const {
    http.setReuse(false);
    http.setTimeout(8000);
    http.addHeader("Content-Type", "application/json");
    http.addHeader("X-Device-Id", deviceId_);
    http.addHeader("X-Device-Secret", deviceSecret_);
}

bool Net::connect(uint32_t timeoutMs) {
    // AP_STA while bridging, so raising the access point later does not tear
    // the station link down.
    WiFi.mode(apSsid_ != nullptr ? WIFI_AP_STA : WIFI_STA);
    WiFi.setAutoReconnect(true);
    WiFi.persistent(false);   // stop rewriting NVS on every begin()
    WiFi.begin(ssid_, password_);

    const uint32_t started = millis();
    while (WiFi.status() != WL_CONNECTED && millis() - started < timeoutMs) {
        delay(250);
    }
    return WiFi.status() == WL_CONNECTED;
}

bool Net::ensureConnected() {
    if (WiFi.status() == WL_CONNECTED) return true;

    if (millis() - lastReconnectAttempt_ < RECONNECT_MIN_GAP_MS) return false;
    lastReconnectAttempt_ = millis();

    WiFi.disconnect();
    if (!connect(8000)) return false;

    // The access point followed the old channel, so it has to come back up on
    // whatever channel the new link landed on.
    if (apSsid_ != nullptr) raiseAp();
    return true;
}

bool Net::raiseAp() {
    // Same channel as the station link. A softAP on a different channel makes
    // the radio hop between the two and both sides start dropping frames.
    const int channel = WiFi.channel();
    if (!WiFi.softAP(apSsid_, apPassword_, channel < 1 ? 1 : channel, 0,
                     apMaxClients_)) {
        bridging_ = false;
        return false;
    }

    // Hand clients the same resolver this node uses, so a hostname in their
    // API_BASE works as well as an address.
    WiFi.softAPConfig(IPAddress(192, 168, 5, 1), IPAddress(192, 168, 5, 1),
                      IPAddress(255, 255, 255, 0), IPAddress(192, 168, 5, 2),
                      WiFi.dnsIP());

    // Without translation the access point is a dead end: clients get an
    // address and reach nothing beyond this node.
    bridging_ = WiFi.AP.enableNAPT(true);
    return bridging_;
}

bool Net::startBridge(const char* apSsid, const char* apPassword,
                      uint8_t maxClients) {
    if (WiFi.status() != WL_CONNECTED) return false;

    apSsid_ = apSsid;
    apPassword_ = apPassword;
    apMaxClients_ = maxClients;
    WiFi.mode(WIFI_AP_STA);
    return raiseAp();
}

void Net::syncTime() {
    // UTC. The server stamps the audit trail; this is for local logging only.
    configTime(0, 0, "pool.ntp.org", "time.nist.gov");
}

void Net::ip(char* out, size_t n) const {
    const IPAddress addr = WiFi.localIP();
    snprintf(out, n, "%u.%u.%u.%u", addr[0], addr[1], addr[2], addr[3]);
}

void Net::bridgeIp(char* out, size_t n) const {
    const IPAddress addr = WiFi.softAPIP();
    snprintf(out, n, "%u.%u.%u.%u", addr[0], addr[1], addr[2], addr[3]);
}

int Net::post(const char* path, const char* body) {
    if (!ensureConnected()) return -1;

    char url[160];
    buildUrl(path, url, sizeof(url));

    HTTPClient http;
    if (!http.begin(url)) return -2;
    addHeaders(http);

    // POST wants a mutable pointer but never writes through it, and the body is
    // ours, so the cast is safe and saves copying it into a String.
    const int code = http.POST(
        const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(body)),
        strlen(body));
    http.end();
    return code;
}

int Net::postParse(const char* path, const char* body, JsonDocument& doc,
                   const JsonDocument& filter) {
    if (!ensureConnected()) return -1;

    char url[160];
    buildUrl(path, url, sizeof(url));

    HTTPClient http;
    if (!http.begin(url)) return -2;
    addHeaders(http);

    const int code = http.POST(
        const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(body)),
        strlen(body));

    if (code == 200) {
        // Parse straight from the socket. No response String is ever built.
        const DeserializationError err =
            deserializeJson(doc, http.getStream(),
                            DeserializationOption::Filter(filter));
        if (err) {
            http.end();
            return -3;
        }
    }

    http.end();
    return code;
}

int Net::get(const char* path, JsonDocument& doc, const JsonDocument& filter) {
    if (!ensureConnected()) return -1;

    char url[160];
    buildUrl(path, url, sizeof(url));

    HTTPClient http;
    if (!http.begin(url)) return -2;
    addHeaders(http);

    const int code = http.GET();

    if (code == 200) {
        const DeserializationError err =
            deserializeJson(doc, http.getStream(),
                            DeserializationOption::Filter(filter));
        if (err) {
            http.end();
            return -3;
        }
    }

    http.end();
    return code;
}
