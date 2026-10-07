# Component tests

One sketch per part, so a fault is found in the part you just wired rather than
somewhere in the whole system. Each sketch is self contained: it declares its
own pins at the top, uses nothing from `door/` or `cabinet/`, and prints its own
wiring notes and instructions over serial when it starts.

Work down the list. Nothing below depends on anything above it except where
said, but the order is the one that isolates faults fastest: it proves the board
before the bus, the bus before the display, and the network before the server.

| # | Sketch | What it proves | Node |
|---|--------|----------------|------|
| 1 | `board_check` | The cable, driver, upload path and monitor all work | both |
| 2 | `i2c_scan` | The display answers, and at which address | both |
| 3 | `lcd` | All four rows are readable, and nothing overruns 20 columns | both |
| 4 | `button_led` | The enroll button debounces and the LED follows it | both |
| 5 | `fingerprint` | The AS608 handshakes, enrolls, searches and deletes, with every prompt on the display too | both |
| 6 | `limit_switch` | The door sensor reads shut and open, and which polarity to configure | cabinet |
| 7 | `relay_lock` | The relay clicks, and which way round to drive it | both |
| 8 | `buzzer` | Each alert pattern, and whether the buzzer is active or passive | cabinet |
| 9 | `wifi` | It joins the network, with enough signal where it will be mounted | both |
| 10 | `backend_ping` | The Rust backend accepts this reader's id and secret | both |
| 11 | `gsm_sms` | The SIM800L registers and can send a text | cabinet |
| 12 | `bench_all` | Everything at once, as one pass or fail table | both |

## Running one

From `esp32/tools`, pass the folder name to the uploader for whichever node the
board is going to be:

```
Upload-Door.cmd fingerprint
Upload-Cabinet.cmd limit_switch
```

That flashes the sketch and leaves the board running it. Open the monitor to
drive it:

```
Monitor-Door.cmd
```

Most sketches take single letter commands, listed when they start and again if
you type anything they do not recognize. Nothing needs to be typed for the
sketches that only watch a pin.

Or open the `.ino` in the Arduino IDE, pick **DOIT ESP32 DEVKIT V1**, and use the
IDE's own monitor at 115200.

## Putting the firmware back

These sketches replace the firmware while they are on the board. Nothing is
lost: reflash it when you are done.

```
Upload-Door.cmd
Upload-Cabinet.cmd
```

The fingerprint sensor is the one thing a reflash does not touch. It keeps its
templates in its own flash, so enrollments survive both these tests and any
number of uploads. The only thing that clears them is `x` in the `fingerprint`
sketch.

## Settings

Tests 9 to 12 need the network and, for 10 and 12, the reader's credentials.
Fill them in at the top of each sketch. They are separate from `Config.h` on
purpose, so trying a setting here never disturbs the real firmware.

Get the device id and secret from the web app under **Admin > Devices**. The
secret is shown once, when you create the device, and cannot be read back.

## Two things that catch everybody

**Power.** Each part wants a different supply, and getting one wrong is the most
common way to lose an afternoon.

| Part | Supply |
|------|--------|
| AS608 fingerprint sensor | `3V3`. It is a 3.3V module and `VIN` will damage it |
| 20x4 display backpack | `VIN`, which is 5V. It will not light properly on 3.3V |
| Solenoid lock | its own 12V supply, through the relay contacts |
| SIM800L | its own 4V supply, 2A capable, with a big capacitor at the module |

The R307 is sold under the same description as the AS608 but is a 5V part, so
check which one you have before moving the red wire. Trying to run the solenoid
or the modem off the board browns it out and reboots it mid scan.

**The display size.** The backpack is the same PCF8574 chip on a 16x2 and a
20x4, so the I2C address says nothing about how big the glass is. The sketches
here are set for 20x4. If rows 3 and 4 stay blank, press `z` in the `lcd` test
to switch, and change `LCD_COLS` and `LCD_ROWS` at the top of the sketches.

**The 2.4 GHz band.** The ESP32 radio has no 5 GHz. If the router publishes one
name for both bands, the join fails with nothing useful in the log. Test 9 lists
every network it can see, which is the quickest way to tell.
