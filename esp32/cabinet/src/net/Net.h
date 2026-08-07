#pragma once

#include <Arduino.h>
#include <ArduinoJson.h>

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
