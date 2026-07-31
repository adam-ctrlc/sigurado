#include "Diag.h"

#include <esp_heap_caps.h>
#include <esp_system.h>

size_t Diag::freeHeap() {
    return ESP.getFreeHeap();
}

size_t Diag::largestBlock() {
    return heap_caps_get_largest_free_block(MALLOC_CAP_8BIT);
}

size_t Diag::minEverFreeHeap() {
    return ESP.getMinFreeHeap();
}

void Diag::begin() {
    baseline_ = freeHeap();
    lastReport_ = millis();
    Serial.printf("[diag] boot reason=%d free=%u largest=%u\n",
                  (int)esp_reset_reason(), (unsigned)baseline_,
                  (unsigned)largestBlock());
}

void Diag::report(const char* tag) {
    Serial.printf("[diag] %s up=%lus free=%u min=%u largest=%u\n",
                  tag, (unsigned long)(millis() / 1000),
                  (unsigned)freeHeap(), (unsigned)minEverFreeHeap(),
                  (unsigned)largestBlock());
}

bool Diag::loop() {
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
