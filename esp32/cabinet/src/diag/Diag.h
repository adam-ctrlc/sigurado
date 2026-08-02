#pragma once

#include <Arduino.h>
#include <esp_heap_caps.h>
#include <esp_system.h>

// Memory health watch.
//
// Fragmentation, not exhaustion, is what kills long-running ESP32 firmware:
// free heap can look fine while the largest contiguous block is too small for
// the next TLS buffer or JSON pool, and the next malloc fails. So we watch the
// largest free block, not just the total.
//
// If it drops below the floor we reboot deliberately. A clean restart that
// takes 3 seconds beats a wedged reader that silently stops logging.
class Diag {
  public:
    Diag(uint32_t reportEveryMs = 60000,
         size_t minFreeBytes = 24576,
         size_t minBlockBytes = 8192)
        : reportEveryMs_(reportEveryMs), minFreeBytes_(minFreeBytes),
          minBlockBytes_(minBlockBytes) {}

    void begin();

    // Call once per loop. Returns false if it decided to reboot (it will not
    // actually return in that case).
    bool loop();

    static size_t freeHeap();
    static size_t largestBlock();
    static size_t minEverFreeHeap();

  private:
    uint32_t reportEveryMs_;
    size_t minFreeBytes_;
    size_t minBlockBytes_;
    uint32_t lastReport_ = 0;
    size_t baseline_ = 0;

    void report(const char* tag);
};

inline size_t Diag::freeHeap() {
    return ESP.getFreeHeap();
}

inline size_t Diag::largestBlock() {
    return heap_caps_get_largest_free_block(MALLOC_CAP_8BIT);
}

inline size_t Diag::minEverFreeHeap() {
    return ESP.getMinFreeHeap();
}

inline void Diag::begin() {
    baseline_ = freeHeap();
    lastReport_ = millis();
    Serial.printf("[diag] boot reason=%d free=%u largest=%u\n",
                  (int)esp_reset_reason(), (unsigned)baseline_,
                  (unsigned)largestBlock());
}

inline void Diag::report(const char* tag) {
    Serial.printf("[diag] %s up=%lus free=%u min=%u largest=%u\n",
                  tag, (unsigned long)(millis() / 1000),
                  (unsigned)freeHeap(), (unsigned)minEverFreeHeap(),
                  (unsigned)largestBlock());
}

inline bool Diag::loop() {
    if (millis() - lastReport_ < reportEveryMs_) return true;
    lastReport_ = millis();

    const size_t freeNow = freeHeap();
    const size_t block = largestBlock();

    if (freeNow < minFreeBytes_ || block < minBlockBytes_) {
        report("LOW MEMORY, restarting");
        Serial.flush();
        delay(200);
        ESP.restart();
        return false;
    }

    report("ok");
    return true;
}
