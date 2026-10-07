#pragma once

#include <Arduino.h>

// These live in a header rather than in the .ino for two reasons, both of them
// things the Arduino build does behind your back.
//
// It generates a prototype for every function and pastes them in near the top
// of the file, above anything you declared yourself, so a function taking an
// Outcome fails to compile with "Outcome has not been declared". Anything
// reached through an #include is already in scope by then.
//
// And FAIL is taken: it is a macro in the ESP-IDF headers the core pulls in.
// An enum class keeps these names to itself, so Fail here collides with
// nothing.
enum class Outcome { Pass, Fail, Skip, Warn };

struct Check {
    const char* name;
    Outcome outcome;
    char detail[64];
};
