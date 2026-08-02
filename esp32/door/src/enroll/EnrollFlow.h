#pragma once

#include <Arduino.h>

#include "../api/ApiClient.h"
#include "../biometric/Biometric.h"
#include "../ui/Panel.h"

// Drives the three-step enrollment handshake:
//
//   1. unknown finger scanned  -> panel says "press ENROLL"
//   2. ENROLL pressed          -> server issues a 6-char code, shown on the LCD
//   3. user types that code into the web app, presses ENROLL again
//      -> the finger is stored locally, then bound to their account server-side
//
// Step 3 is deliberately gated on a second button press: the device has no way
// to know when the website confirmed the code, so the user tells it.
class EnrollFlow {
  public:
    enum class Stage { Idle, AwaitWebCode };

    EnrollFlow(ApiClient& api, Biometric& bio, Panel& panel)
        : api_(api), bio_(bio), panel_(panel) {}

    Stage stage() const { return stage_; }
    const char* code() const { return code_; }

    // Call on each ENROLL button press.
    void onButton();
    void reset();

  private:
    ApiClient& api_;
    Biometric& bio_;
    Panel& panel_;
    Stage stage_ = Stage::Idle;
    char code_[12] = {0};

    void requestCode();
    void bindFinger();
};

namespace enroll_detail {

constexpr uint32_t HOLD_MS = 2500;

// The sensor calls back with a plain function pointer, so the panel it should
// write to has to be reachable without a captured `this`.
inline Panel* g_panel = nullptr;

inline void stepPrompt(const char* text) {
    if (g_panel) g_panel->show(text, "hold still");
}

}  // namespace enroll_detail

using enroll_detail::HOLD_MS;
using enroll_detail::g_panel;
using enroll_detail::stepPrompt;

inline void EnrollFlow::onButton() {
    if (stage_ == Stage::Idle) {
        requestCode();
    } else {
        bindFinger();
    }
}

inline void EnrollFlow::reset() {
    stage_ = Stage::Idle;
    code_[0] = '\0';
}

inline void EnrollFlow::requestCode() {
    panel_.show("Getting a code", "one moment");

    const EnrollCodeResult res = api_.requestEnrollCode();
    if (!res.ok) {
        panel_.showf("Not started yet", "code %d", res.httpCode);
        panel_.blink(3);
        delay(HOLD_MS);
        return;
    }

    strncpy(code_, res.code, sizeof(code_) - 1);
    code_[sizeof(code_) - 1] = '\0';
    stage_ = Stage::AwaitWebCode;

    char line[17];
    snprintf(line, sizeof(line), "Code %s", code_);
    panel_.show(line, "Type it on site");
}

inline void EnrollFlow::bindFinger() {
    const int slot = bio_.nextFreeSlot();
    if (slot < 0) {
        panel_.show("Sensor is full", "ask an admin");
        delay(HOLD_MS);
        reset();
        return;
    }

    g_panel = &panel_;
    const bool stored = bio_.enroll(static_cast<uint16_t>(slot), stepPrompt);
    g_panel = nullptr;

    if (!stored) {
        panel_.show("Did not read", "please try again");
        panel_.blink(3);
        delay(HOLD_MS);
        reset();
        return;
    }

    panel_.show("Saving...", "one moment");

    char token[32];
    api_.tokenFor(static_cast<uint16_t>(slot), token, sizeof(token));
    const BindResult res = api_.bind(token);

    if (res.bound) {
        panel_.show("All set", res.userName);
        panel_.blink(2, 80);
    } else {
        // do not leave an orphan template if the server refused the bind
        bio_.erase(static_cast<uint16_t>(slot));
        panel_.show("Not saved yet",
                    res.message[0] ? res.message : "enter code first");
        panel_.blink(4);
    }

    delay(HOLD_MS);
    reset();
}
