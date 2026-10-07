// TEST 8 of 12: the buzzer. Cabinet node only.
//
// Wiring:
//   buzzer positive to GPIO 19
//   buzzer negative to GND
//
// Two kinds get sold as "buzzer" and they behave differently:
//   active  has its own oscillator. Any steady voltage makes its one note.
//   passive is a small speaker. It needs a square wave, and the pitch is yours
//           to choose.
// Press 'd' to find out which one you have.
//
// Expect: each pattern the firmware uses, on demand, so you can hear whether
// the alarm is loud enough to notice from across the stockroom without being
// unpleasant to work beside.
//
// Monitor commands:
//   d  detect which kind of buzzer is fitted
//   1  granted, one short rise
//   2  refused, two low notes
//   3  door left open warning
//   4  alarm
//   5  enrollment saved
//   s  silence

static const uint8_t PIN_BUZZER = 19;

// LEDC channel. Channel 0 is fine here: nothing else on this node uses PWM.
static const uint8_t PWM_CHANNEL = 0;

bool passive = true;

void note(uint16_t hz, uint16_t ms) {
    if (passive) {
        ledcWriteTone(PIN_BUZZER, hz);
        delay(ms);
        ledcWriteTone(PIN_BUZZER, 0);
    } else {
        digitalWrite(PIN_BUZZER, HIGH);
        delay(ms);
        digitalWrite(PIN_BUZZER, LOW);
    }
}

void rest(uint16_t ms) {
    delay(ms);
}

void silence() {
    if (passive) {
        ledcWriteTone(PIN_BUZZER, 0);
    } else {
        digitalWrite(PIN_BUZZER, LOW);
    }
}

void setupPin() {
    silence();
    if (passive) {
        ledcAttach(PIN_BUZZER, 2000, 10);
        ledcWriteTone(PIN_BUZZER, 0);
    } else {
        ledcDetach(PIN_BUZZER);
        pinMode(PIN_BUZZER, OUTPUT);
        digitalWrite(PIN_BUZZER, LOW);
    }
}

void detect() {
    Serial.println(F("Listen. Two tests, three seconds apart."));

    Serial.println(F("  1. steady voltage, no oscillation..."));
    ledcDetach(PIN_BUZZER);
    pinMode(PIN_BUZZER, OUTPUT);
    digitalWrite(PIN_BUZZER, HIGH);
    delay(800);
    digitalWrite(PIN_BUZZER, LOW);

    delay(1200);

    Serial.println(F("  2. a 2 kHz square wave..."));
    ledcAttach(PIN_BUZZER, 2000, 10);
    ledcWriteTone(PIN_BUZZER, 2000);
    delay(800);
    ledcWriteTone(PIN_BUZZER, 0);

    Serial.println();
    Serial.println(F("Heard a tone on test 1?  active buzzer. Press 'a'."));
    Serial.println(F("Only on test 2?          passive buzzer. Press 'p'."));
    Serial.println(F("Neither?                 check the polarity and the GND."));
}

// Granted. Rising, so it reads as a door opening rather than an error.
void granted() {
    note(1200, 90);
    rest(40);
    note(1800, 140);
}

// Refused. Low and short. Never harsh: being turned away in front of a queue is
// bad enough without the whole room hearing about it.
void refused() {
    note(500, 160);
    rest(70);
    note(400, 220);
}

void doorWarning() {
    for (uint8_t i = 0; i < 2; i++) {
        note(900, 120);
        rest(120);
    }
}

// Alarm. This one is meant to be annoying, and only fires when the cabinet has
// stood open well past the point of an honest mistake.
void alarm() {
    for (uint8_t i = 0; i < 6; i++) {
        note(2200, 180);
        rest(90);
        note(1500, 180);
        rest(90);
    }
}

void saved() {
    note(1000, 90);
    rest(40);
    note(1400, 90);
    rest(40);
    note(1900, 200);
}

void help() {
    Serial.println();
    Serial.println(F("d detect  1 granted  2 refused  3 warning  4 alarm  5 saved  s stop"));
    Serial.print(F("fitted: "));
    Serial.println(passive ? F("passive (tones)") : F("active (one note)"));
}

void setup() {
    Serial.begin(115200);
    delay(300);

    setupPin();

    Serial.println();
    Serial.println(F("Buzzer test."));
    Serial.println(F("Press 'd' first if you do not know which kind you have."));
    help();
}

void loop() {
    if (!Serial.available()) {
        return;
    }

    const char c = Serial.read();
    switch (c) {
        case 'd':
            detect();
            break;
        case 'a':
            passive = false;
            setupPin();
            Serial.println(F("  set to active."));
            break;
        case 'p':
            passive = true;
            setupPin();
            Serial.println(F("  set to passive."));
            break;
        case '1':
            Serial.println(F("granted"));
            granted();
            break;
        case '2':
            Serial.println(F("refused"));
            refused();
            break;
        case '3':
            Serial.println(F("door left open"));
            doorWarning();
            break;
        case '4':
            Serial.println(F("alarm"));
            alarm();
            break;
        case '5':
            Serial.println(F("enrollment saved"));
            saved();
            break;
        case 's':
            silence();
            Serial.println(F("quiet."));
            break;
        case '\n':
        case '\r':
            break;
        default:
            help();
            break;
    }
}
