# Tools

Double-click the `.cmd` files. Each one is a thin wrapper that runs the matching
PowerShell script in `scripts/` with the execution policy bypassed for that one
call, so nothing has to be changed machine wide.

| File | What it does |
|------|--------------|
| `Find-Board.cmd` | Lists the boards it can see and says why each port was picked |
| `Upload-Door.cmd` | Compiles and flashes `door/` |
| `Upload-Cabinet.cmd` | Compiles and flashes `cabinet/` |
| `Monitor-Door.cmd` | Serial monitor on the door node |
| `Monitor-Cabinet.cmd` | Serial monitor on the cabinet node |

Both uploaders take an optional sketch name to send one of the component tests
in `testing/` instead of the firmware:

```
Upload-Door.cmd fingerprint
Upload-Cabinet.cmd limit_switch
```

## Finding the right board

You never type a COM number. `Find-Board.ps1` reads the USB vendor and product
id behind every serial port and only considers the chips an ESP32 board actually
uses: CP210x, CH340, CH9102, FTDI, and Espressif's own native USB. A laptop with
a paired phone has two or three Bluetooth COM ports that look like boards to
anything that only counts ports, and those are excluded.

The door and cabinet nodes are the same hardware, so USB cannot tell them apart.

- One board plugged in: it is taken as whichever node you asked for.
- Two plugged in, and one already known: the other is deduced, no question asked.
- Two plugged in and neither known: you are asked once, and the answer is saved
  in `scripts/nodes.json` against the board's USB instance id, so it survives
  Windows renumbering the ports.

`nodes.json` is local to your machine and is not committed. Delete it, or run
`scripts\Find-Board.ps1 -Node door -Forget`, to be asked again.

## Requirements

`arduino-cli` on the PATH:

```
winget install ArduinoSA.CLI
```

then the board support and the three libraries:

```
arduino-cli core install esp32:esp32
arduino-cli lib install "Adafruit Fingerprint Sensor Library"
arduino-cli lib install "LiquidCrystal I2C"
arduino-cli lib install ArduinoJson
```

## Notes

The board is **DOIT ESP32 DEVKIT V1**, FQBN `esp32:esp32:esp32doit-devkit-v1`.
It has no PSRAM or JTAG entries in the Tools menu, so there is nothing there to
turn off. Those options only appear for the S3 boards.

Uploading starts at 921600 baud and retries once at 115200 if that fails, which
is the usual fix on a CH340 with a long lead rather than anything to do with the
wiring.

A monitor window holds the port open, and an upload cannot then claim it. Close
it with Ctrl+C first. If an upload fails part way through, that is the first
thing to check.

Run any script directly for the full help:

```
Get-Help .\scripts\Find-Board.ps1 -Full
```
