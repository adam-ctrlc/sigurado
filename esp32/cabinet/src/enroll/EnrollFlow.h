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
