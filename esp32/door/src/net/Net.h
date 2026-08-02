#pragma once

#include <Arduino.h>
#include <ArduinoJson.h>
#include <HTTPClient.h>
#include <WiFi.h>
#include <time.h>

// WiFi association, NTP, and device-authenticated requests.
//
// Every request carries the X-Device-Id / X-Device-Secret pair. A device never
// holds a user JWT, so a stolen reader cannot impersonate a person.
//
// MEMORY: nothing here returns an Arduino String. Request bodies are caller
// owned char buffers and the response is parsed straight off the socket into a
// filtered JsonDocument, so a long-running node never fragments its heap on
// response strings.
class Net {
  public:
    Net(const char* ssid, const char* password, const char* apiBase,
        const char* deviceId, const char* deviceSecret)
        : ssid_(ssid), password_(password), apiBase_(apiBase),
          deviceId_(deviceId), deviceSecret_(deviceSecret) {}

    bool connect(uint32_t timeoutMs = 20000);
    bool ensureConnected();
    void syncTime();
    void ip(char* out, size_t n) const;

    // Share this node's connection with the other one.
    //
    // The ESP32 can hold a station link and run an access point at the same
    // time, and with address translation switched on, whatever joins the access
    // point reaches the network behind it. That lets the second node sit outside
    // WiFi range and still talk to the server through this one.
    //
    // Call it only once the station link is up: an access point has to sit on
    // the same radio channel as the link it borrows.
    bool startBridge(const char* apSsid, const char* apPassword,
                     uint8_t maxClients = 4);
    bool bridging() const { return bridging_; }
    void bridgeIp(char* out, size_t n) const;

    // GET, parsed like postParse below.
    int get(const char* path, JsonDocument& doc, const JsonDocument& filter);

    // POST with no interest in the response body.
    int post(const char* path, const char* body);

    // POST and parse the response into `doc`. `filter` keeps only the fields we
    // care about, which bounds the document to a few hundred bytes.
    // Returns the HTTP status code, or negative if the request never went out.
    int postParse(const char* path, const char* body, JsonDocument& doc,
                  const JsonDocument& filter);

  private:
    const char* ssid_;
    const char* password_;
    const char* apiBase_;
    const char* deviceId_;
    const char* deviceSecret_;
    uint32_t lastReconnectAttempt_ = 0;
    bool bridging_ = false;
    const char* apSsid_ = nullptr;
    const char* apPassword_ = nullptr;
    uint8_t apMaxClients_ = 4;

    void buildUrl(const char* path, char* out, size_t n) const;
    void addHeaders(class HTTPClient& http) const;
    bool raiseAp();
};

// Rate limit reconnect storms: each WiFi.begin() churns the heap.
constexpr uint32_t RECONNECT_MIN_GAP_MS = 5000;

inline void Net::buildUrl(const char* path, char* out, size_t n) const {
    snprintf(out, n, "%s%s", apiBase_, path);
}

inline void Net::addHeaders(HTTPClient& http) const {
    http.setReuse(false);
    http.setTimeout(8000);
    http.addHeader("Content-Type", "application/json");
    http.addHeader("X-Device-Id", deviceId_);
    http.addHeader("X-Device-Secret", deviceSecret_);
}

inline bool Net::connect(uint32_t timeoutMs) {
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

inline bool Net::ensureConnected() {
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

inline bool Net::raiseAp() {
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

inline bool Net::startBridge(const char* apSsid, const char* apPassword,
                      uint8_t maxClients) {
    if (WiFi.status() != WL_CONNECTED) return false;

    apSsid_ = apSsid;
    apPassword_ = apPassword;
    apMaxClients_ = maxClients;
    WiFi.mode(WIFI_AP_STA);
    return raiseAp();
}

inline void Net::syncTime() {
    // UTC. The server stamps the audit trail; this is for local logging only.
    configTime(0, 0, "pool.ntp.org", "time.nist.gov");
}

inline void Net::ip(char* out, size_t n) const {
    const IPAddress addr = WiFi.localIP();
    snprintf(out, n, "%u.%u.%u.%u", addr[0], addr[1], addr[2], addr[3]);
}

inline void Net::bridgeIp(char* out, size_t n) const {
    const IPAddress addr = WiFi.softAPIP();
    snprintf(out, n, "%u.%u.%u.%u", addr[0], addr[1], addr[2], addr[3]);
}

inline int Net::post(const char* path, const char* body) {
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

inline int Net::postParse(const char* path, const char* body, JsonDocument& doc,
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

inline int Net::get(const char* path, JsonDocument& doc, const JsonDocument& filter) {
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
