#include "EnrollFlow.h"

namespace {

const uint32_t HOLD_MS = 2500;

Panel* g_panel = nullptr;
void stepPrompt(const char* text) {
    if (g_panel) g_panel->show(text, "hold still");
}

}  // namespace

void EnrollFlow::onButton() {
    if (stage_ == Stage::Idle) {
        requestCode();
    } else {
        bindFinger();
    }
}

void EnrollFlow::reset() {
    stage_ = Stage::Idle;
    code_[0] = '\0';
}

void EnrollFlow::requestCode() {
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

void EnrollFlow::bindFinger() {
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
