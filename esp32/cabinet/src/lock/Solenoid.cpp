#include "Solenoid.h"

void Solenoid::begin() {
    pinMode(pin_, OUTPUT);
    lock();
}

void Solenoid::lock() {
    digitalWrite(pin_, activeLow_ ? HIGH : LOW);
}

void Solenoid::release() {
    digitalWrite(pin_, activeLow_ ? LOW : HIGH);
    delay(holdMs_);
    lock();
}
