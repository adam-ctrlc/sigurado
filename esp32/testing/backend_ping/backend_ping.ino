// TEST 10 of 12: talking to the Rust backend.
//
// Wiring: none. This needs test 9 passing first.
//
// Fill in all five settings below. Get the device id and secret from the web
// app under Admin > Devices, where you register a reader. The secret is shown
// once, at the moment you create it, and never again, so paste it straight in.
//
// Expect: HTTP 200 from the heartbeat, and the reader turning green on the
// Devices page while this runs.
//
// What the status codes mean here:
//   200  everything is right
//   401  the id and secret do not match a registered device
//   404  API_BASE is wrong, most likely missing the /api on the end
//   -1   no answer at all: wrong IP, wrong port, or the firewall
//
// Windows blocks incoming connections to a new program by default, so the very
// first time you run the backend it will not be reachable from the reader even
// though it works fine in your own browser. Allow it on private networks.
//
// Monitor commands:
//   h  heartbeat
//   s  a scan with a made-up finger token, to see a refusal come back
//   r  raw GET of the base URL, to prove the server is even there
//   l  loop a heartbeat every 5 seconds, like the real firmware

#include <HTTPClient.h>
#include <WiFi.h>

static const char* WIFI_SSID = "YOUR_WIFI";
static const char* WIFI_PASS = "YOUR_WIFI_PASSWORD";

// No trailing slash.
static const char* API_BASE = "http://192.168.1.50:8080/api";

static const char* DEVICE_ID = "PASTE_DEVICE_ID";
static const char* DEVICE_SECRET = "PASTE_DEVICE_SECRET";

static const uint8_t PIN_LED = 2;

bool looping = false;
uint32_t lastBeat = 0;

bool ensureWifi() {
    if (WiFi.status() == WL_CONNECTED) {
        return true;
    }

    Serial.print(F("joining wifi..."));
    WiFi.mode(WIFI_STA);
    WiFi.begin(WIFI_SSID, WIFI_PASS);

    const uint32_t started = millis();
    while (WiFi.status() != WL_CONNECTED && millis() - started < 20000) {
        delay(250);
    }

    if (WiFi.status() != WL_CONNECTED) {
        Serial.println(F(" failed. Run the wifi test first."));
        return false;
    }

    Serial.print(F(" "));
    Serial.println(WiFi.localIP());
    return true;
}

void explain(int code) {
    switch (code) {
        case 200:
            Serial.println(F("  the server accepted this reader."));
            break;
        case 401:
            Serial.println(F("  rejected. The id or the secret is wrong. A secret"));
            Serial.println(F("  cannot be read back, so if you lost it, delete the"));
            Serial.println(F("  device on the website and register it again."));
            break;
        case 403:
            Serial.println(F("  the device is registered but disabled. Turn it back"));
            Serial.println(F("  on under Admin > Devices."));
            break;
        case 404:
            Serial.println(F("  no such endpoint. Check API_BASE ends with /api and"));
            Serial.println(F("  has no trailing slash."));
            break;
        case -1:
            Serial.println(F("  no answer. Check the IP and port, that the backend"));
            Serial.println(F("  is running, and that Windows Firewall allows it on"));
            Serial.println(F("  private networks."));
            break;
        default:
            break;
    }
}

int post(const char* path, const char* body, bool quiet = false) {
    if (!ensureWifi()) {
        return -1;
    }

    char url[160];
    snprintf(url, sizeof(url), "%s%s", API_BASE, path);

    HTTPClient http;
    http.setTimeout(8000);
    if (!http.begin(url)) {
        Serial.println(F("  malformed URL."));
        return -1;
    }

    http.addHeader("Content-Type", "application/json");
    http.addHeader("X-Device-Id", DEVICE_ID);
    http.addHeader("X-Device-Secret", DEVICE_SECRET);

    if (!quiet) {
        Serial.print(F("POST "));
        Serial.println(url);
        Serial.print(F("  body: "));
        Serial.println(body);
    }

    const uint32_t started = millis();
    const int code = http.POST((uint8_t*)body, strlen(body));
    const uint32_t took = millis() - started;

    Serial.print(F("  -> "));
    Serial.print(code);
    Serial.print(F("  in "));
    Serial.print(took);
    Serial.println(F(" ms"));

    if (code > 0) {
        const String payload = http.getString();
        if (payload.length() > 0) {
            Serial.print(F("  body: "));
            Serial.println(payload);
        }
    }

    if (!quiet) {
        explain(code);
    }

    http.end();
    digitalWrite(PIN_LED, code == 200 ? HIGH : LOW);
    return code;
}

void rawGet() {
    if (!ensureWifi()) {
        return;
    }

    HTTPClient http;
    http.setTimeout(8000);
    http.begin(API_BASE);

    Serial.print(F("GET "));
    Serial.println(API_BASE);

    const int code = http.GET();
    Serial.print(F("  -> "));
    Serial.println(code);

    if (code > 0) {
        Serial.println(F("  the server is reachable. Any code at all proves the"));
        Serial.println(F("  network path works, even a 404."));
    } else {
        explain(-1);
    }

    http.end();
}

void help() {
    Serial.println();
    Serial.println(F("h heartbeat   s fake scan   r raw get   l loop on/off"));
}

void setup() {
    Serial.begin(115200);
    delay(300);

    pinMode(PIN_LED, OUTPUT);
    digitalWrite(PIN_LED, LOW);

    Serial.println();
    Serial.println(F("Backend test."));

    if (strcmp(DEVICE_ID, "PASTE_DEVICE_ID") == 0) {
        Serial.println(F("DEVICE_ID is still the placeholder. Register a reader"));
        Serial.println(F("under Admin > Devices and paste the id and secret in."));
    }

    ensureWifi();
    post("/device/heartbeat", "{\"firmware\":\"bench-test\"}");
    help();
}

void loop() {
    if (looping && millis() - lastBeat >= 5000) {
        lastBeat = millis();
        post("/device/heartbeat", "{\"firmware\":\"bench-test\"}", true);
    }

    if (!Serial.available()) {
        return;
    }

    const char c = Serial.read();
    switch (c) {
        case 'h':
            post("/device/heartbeat", "{\"firmware\":\"bench-test\"}");
            break;
        case 's':
            // A token no sensor would ever produce, so this is always a refusal.
            // A 200 with unlock false is the right answer, not an error.
            post("/device/scan", "{\"finger_token\":\"bench:9999\"}");
            break;
        case 'r':
            rawGet();
            break;
        case 'l':
            looping = !looping;
            Serial.print(F("  looping "));
            Serial.println(looping ? F("on") : F("off"));
            break;
        case '\n':
        case '\r':
            break;
        default:
            help();
            break;
    }
}
