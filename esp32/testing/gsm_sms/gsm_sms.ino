// TEST 11 of 12: the SIM800L modem. Cabinet node only.
//
// The power supply is what makes or breaks this module, so read this first.
//
// The SIM800L pulls up to 2 amps in short bursts every time it talks to the
// tower. The ESP32's regulator cannot supply that. Run the modem from its own
// 4.0V supply (3.7 to 4.2V, and NOT 5V), with a large capacitor, 1000uF or
// more, right at the module's pins. A bench supply, a lithium cell, or a buck
// converter rated 2A all work. USB power does not.
//
// The single most common symptom of an underfed module is that it answers AT
// fine and then reboots the moment it tries to register on the network.
//
// Wiring:
//   modem VCC  to the 4V supply
//   modem GND  to that supply's ground, AND to the ESP32 GND
//   modem TXD  to GPIO 26
//   modem RXD  to GPIO 27, through a divider or a level shifter
//   modem RST  to GPIO 25 (optional)
//
// That divider is not optional. The SIM800L's receive pin is not 3.3V tolerant
// despite what the listings say. Two resistors, 1k from GPIO 27 to the modem
// RXD and 2k from modem RXD to ground, is enough.
//
// The SIM must have no PIN lock and some credit. Put it in a phone first and
// send one text to prove both.
//
// Expect: OK to AT, a CSQ above 10, and CREG showing 1 or 5.
//
// Monitor commands:
//   a  AT, the basic are-you-there
//   i  module identity and firmware
//   q  signal strength
//   n  network registration
//   p  SIM PIN status
//   s  send a text (asks for the number and the message)
//   r  hardware reset
//   >  pass anything else straight through as an AT command

static const uint8_t PIN_GSM_RX = 26;
static const uint8_t PIN_GSM_TX = 27;
static const uint8_t PIN_GSM_RST = 25;
static const uint32_t GSM_BAUD = 9600;

HardwareSerial gsm(1);

// Reads whatever the modem says for a while and prints it. Returns true if the
// wanted word showed up.
bool waitFor(const char* wanted, uint32_t timeoutMs, bool echo = true) {
    String buffer;
    const uint32_t started = millis();

    while (millis() - started < timeoutMs) {
        while (gsm.available()) {
            const char c = gsm.read();
            buffer += c;
            if (echo) {
                Serial.write(c);
            }
        }
        if (wanted != nullptr && buffer.indexOf(wanted) >= 0) {
            return true;
        }
        delay(10);
    }
    return false;
}

void send(const char* command, uint32_t timeoutMs = 3000) {
    Serial.print(F("> "));
    Serial.println(command);
    gsm.println(command);
    waitFor("OK", timeoutMs);
    Serial.println();
}

void hardReset() {
    if (PIN_GSM_RST == 255) {
        Serial.println(F("  no reset line wired."));
        return;
    }
    Serial.println(F("  pulling reset low for 150 ms..."));
    pinMode(PIN_GSM_RST, OUTPUT);
    digitalWrite(PIN_GSM_RST, LOW);
    delay(150);
    digitalWrite(PIN_GSM_RST, HIGH);
    Serial.println(F("  give it 10 seconds to find the network."));
}

void signalStrength() {
    Serial.println(F("> AT+CSQ"));
    gsm.println("AT+CSQ");

    String buffer;
    const uint32_t started = millis();
    while (millis() - started < 3000) {
        while (gsm.available()) {
            buffer += (char)gsm.read();
        }
        if (buffer.indexOf("OK") >= 0) {
            break;
        }
        delay(10);
    }

    Serial.print(buffer);

    const int at = buffer.indexOf("+CSQ:");
    if (at < 0) {
        Serial.println(F("  no reply. Check the wiring and the power."));
        return;
    }

    const int rssi = buffer.substring(at + 6, buffer.indexOf(',', at)).toInt();
    Serial.print(F("  rssi value "));
    Serial.print(rssi);
    Serial.print(F(": "));

    if (rssi == 99) {
        Serial.println(F("unknown. No usable signal yet."));
    } else if (rssi < 5) {
        Serial.println(F("far too weak. Move the antenna away from the metal."));
    } else if (rssi < 10) {
        Serial.println(F("marginal. Texts will be slow and some will fail."));
    } else if (rssi < 20) {
        Serial.println(F("workable."));
    } else {
        Serial.println(F("good."));
    }
    Serial.println();
}

void registration() {
    Serial.println(F("> AT+CREG?"));
    gsm.println("AT+CREG?");

    String buffer;
    const uint32_t started = millis();
    while (millis() - started < 3000) {
        while (gsm.available()) {
            buffer += (char)gsm.read();
        }
        if (buffer.indexOf("OK") >= 0) {
            break;
        }
        delay(10);
    }

    Serial.print(buffer);

    const int at = buffer.indexOf("+CREG:");
    if (at < 0) {
        Serial.println(F("  no reply."));
        return;
    }

    const int comma = buffer.indexOf(',', at);
    const int state = buffer.substring(comma + 1, comma + 2).toInt();

    Serial.print(F("  state "));
    Serial.print(state);
    Serial.print(F(": "));
    switch (state) {
        case 0:
            Serial.println(F("not registered and not looking. Check the SIM."));
            break;
        case 1:
            Serial.println(F("registered on the home network. Good."));
            break;
        case 2:
            Serial.println(F("still searching. Wait, then ask again."));
            break;
        case 3:
            Serial.println(F("registration refused. Usually a dead or barred SIM."));
            break;
        case 5:
            Serial.println(F("registered, roaming. Good, but texts may cost more."));
            break;
        default:
            Serial.println(F("unknown."));
            break;
    }
    Serial.println();
}

String readLine(const __FlashStringHelper* prompt, uint32_t timeoutMs) {
    Serial.println(prompt);
    while (Serial.available()) {
        Serial.read();
    }

    const uint32_t started = millis();
    while (!Serial.available() && millis() - started < timeoutMs) {
        delay(20);
    }

    String line = Serial.readStringUntil('\n');
    line.trim();
    return line;
}

void sendSms() {
    const String number = readLine(F("number, in full international form, e.g. +639171234567:"), 30000);
    if (number.length() < 5) {
        Serial.println(F("  cancelled."));
        return;
    }

    const String body = readLine(F("message:"), 60000);
    if (body.length() == 0) {
        Serial.println(F("  cancelled."));
        return;
    }

    Serial.println(F("  switching to text mode..."));
    send("AT+CMGF=1");

    Serial.print(F("  sending to "));
    Serial.println(number);

    gsm.print("AT+CMGS=\"");
    gsm.print(number);
    gsm.println("\"");

    if (!waitFor(">", 5000)) {
        Serial.println(F("  the modem never asked for the message body."));
        return;
    }

    gsm.print(body);
    // Ctrl-Z is what actually sends it.
    gsm.write(26);

    Serial.println(F("  sent, waiting for the network to confirm (up to 60s)..."));
    if (waitFor("+CMGS", 60000)) {
        Serial.println();
        Serial.println(F("  accepted by the network."));
    } else {
        Serial.println();
        Serial.println(F("  no confirmation. Check credit, signal, and the power"));
        Serial.println(F("  supply. A module that resets here is underfed."));
    }
    Serial.println();
}

void help() {
    Serial.println();
    Serial.println(F("a AT   i identity   q signal   n network   p sim pin"));
    Serial.println(F("s send text   r reset   anything else is sent as AT"));
}

void setup() {
    Serial.begin(115200);
    delay(300);

    gsm.begin(GSM_BAUD, SERIAL_8N1, PIN_GSM_RX, PIN_GSM_TX);

    if (PIN_GSM_RST != 255) {
        pinMode(PIN_GSM_RST, OUTPUT);
        digitalWrite(PIN_GSM_RST, HIGH);
    }

    Serial.println();
    Serial.println(F("SIM800L test."));
    Serial.println(F("Give it 10 seconds after power-up before expecting a reply."));
    Serial.println(F("The status LED should blink slowly once it has registered:"));
    Serial.println(F("fast blinking means it is still searching."));

    delay(3000);
    send("AT");
    send("ATE0");
    help();
}

void loop() {
    // Anything the modem says on its own, unasked, still gets shown.
    while (gsm.available()) {
        Serial.write(gsm.read());
    }

    if (!Serial.available()) {
        return;
    }

    const char c = Serial.peek();
    switch (c) {
        case 'a':
            Serial.read();
            send("AT");
            return;
        case 'i':
            Serial.read();
            send("ATI");
            send("AT+CCID");
            send("AT+COPS?");
            return;
        case 'q':
            Serial.read();
            signalStrength();
            return;
        case 'n':
            Serial.read();
            registration();
            return;
        case 'p':
            Serial.read();
            send("AT+CPIN?");
            return;
        case 's':
            Serial.read();
            sendSms();
            return;
        case 'r':
            Serial.read();
            hardReset();
            return;
        case '\n':
        case '\r':
            Serial.read();
            return;
        default:
            break;
    }

    String line = Serial.readStringUntil('\n');
    line.trim();
    if (line.length() > 0) {
        Serial.print(F("> "));
        Serial.println(line);
        gsm.println(line);
    }
}
