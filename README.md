# Sigurado

**Two scans. One person. No mystery.**

Sigurado is a dual-biometric access and materials accountability system for a school
laboratory stockroom. A fingerprint at the door opens a short window; the cabinet then
opens only for that same person, while that window is still live. Everything is recorded
with an identity, a reader, and a server timestamp, and shown on a live web console.

The name is Filipino for "sure", which is the point: at any moment you can say who opened
the cabinet and what they took.

---

## Why it works this way

Most fingerprint locks answer one question: is this a registered finger? That still lets a
stranger walk in behind somebody and open the cabinet a minute later. Sigurado answers a
harder one: **is this the person who just came through the door?**

1. **Scan at the door.** The lock releases and a timed window opens, belonging to one
   person.
2. **Scan at the cabinet, inside that window.** The reader checks that the finger it just
   read belongs to the person the door let in. Anything else is refused and recorded as a
   tailgating attempt.
3. **Record what you took.** The cabinet shows a one-time code on its screen as it opens.
   The website will not accept a checkout without that code, so a written record always
   corresponds to a real, physical opening.

The system separates two kinds of fact and never confuses them. **Reader events** come
from the hardware and cannot be edited by anyone, including an administrator.
**Checkouts** are claims typed in by a person, trustworthy only because they require the
code from the opening. The gap between the two is where the interesting questions live,
which is what the flags page is for.

## What it does

- **Sequential dual biometrics.** Door then cabinet, with a session that belongs to one
  person and expires on its own.
- **Enrollment with claiming.** A reader captures a fingerprint and shows a one-time code.
  The finger belongs to nobody until somebody enters that code on the website, so a
  template can never be silently bound to the wrong account.
- **Materials checkout.** An item and quantity table, an optional photo, and the cabinet's
  code as proof the opening happened.
- **Live audit trail.** Every scan, unlock, refusal and enrollment, streamed over
  server-sent events, searchable and filterable from the address bar.
- **Flags.** Eleven checks run over the trail and surface what does not add up: an opening
  with no record behind it, tailgating, guessed checkout codes, unregistered fingers tried
  repeatedly, openings outside lab hours, disabled accounts still trying, readers that have
  gone quiet, enrollment codes nobody ever claimed, and more.
- **Sheet.** The whole trail as a spreadsheet, with lettered columns, numbered rows, a
  frozen header and CSV export. Optionally mirrored to Google Sheets.
- **SMS alerts.** A SIM800L on the cabinet node sends a text when the cabinet opens or
  access is refused. The server queues; the node sends.
- **Roles.** Students see their own records, faculty see everything and change nothing,
  administrators run the room. An administrator cannot disable, delete or demote
  themselves.
- **Guides.** Numbered, step-by-step instructions built into the app for each role.

## Repository layout

```
backend/     Rust API: axum 0.8, SeaORM 1.1, SQLite, JWT, argon2
frontend/    SvelteKit 2 + Svelte 5 runes, Tailwind CSS v4, shadcn-svelte
esp32/       Firmware for both reader nodes (Arduino, ESP32 DevKit v1)
visualize/   Parametric OpenSCAD model of the cabinet and the scanner enclosure
```

## Hardware

Two ESP32 nodes, one at the door and one on the cabinet. Nothing exotic.

| Part | Notes |
| --- | --- |
| ESP32 DevKit v1, 30-pin | One per node. PSRAM and JTAG disabled in the IDE. |
| R307 or AS608 fingerprint sensor | One per node, on UART2. Templates live on the sensor. |
| 16x2 I2C LCD | Prompts, and the checkout code on the cabinet node. |
| Relay plus 12V solenoid | Fail-secure; the coil never runs off the ESP32 rails. |
| Lever limit switch | Reports whether the door is really shut. Debounced in firmware. |
| SIM800L | Cabinet node only, on UART1, with its own 4V supply. |
| Buzzer, LED, push button | Enrollment and the door-left-open reminder. |

Wiring for each node is documented at the top of its sketch. Matching is done on the
sensor itself, so slot numbers are local to a reader; that is why a person enrolls at the
door and at the cabinet separately, and why finger tokens are namespaced per device.

If the door node cannot reach the router, the cabinet node can share its connection:
set `SHARE_WIFI` in the cabinet config and point the door node at its SSID.

## Running it

### Backend

```bash
cd backend
cp .env.example .env          # then fill in DATABASE_URL and JWT_SECRET
cargo run -p migration -- up  # migrations do not run at startup
cargo run --bin seed          # creates the first administrator
cargo run --bin backend
```

Listens on `127.0.0.1:8080` by default. The SQLite file lives in `backend/db/`,
which is where `.env.example` points and the only place worth backing up. The
server creates that folder if it is missing, so pointing `DATABASE_URL` somewhere
new needs no `mkdir` first.

### Frontend

```bash
cd frontend
pnpm install
pnpm dev
```

Serves on `localhost:5173` and expects the API at `localhost:8080`.

### Firmware

Open `esp32/door/door.ino` or `esp32/cabinet/cabinet.ino` in the Arduino IDE. Board:
**ESP32 Dev Module**. Libraries: Adafruit Fingerprint Sensor Library, LiquidCrystal I2C,
ArduinoJson v7. Pair each node from **Devices** in the web app, then paste the device id
and secret it gives you into `src/Config.h`. The secret is shown once.

Both sketches compile clean against esp32 core 3.3.11, at roughly 81% of the default
1.2MB app partition.

## Configuration

All backend settings live in `backend/.env`; see `.env.example`. Notable ones:

| Variable | Default | What it does |
| --- | --- | --- |
| `SESSION_TTL_SECS` | 120 | How long the door window stays open. |
| `ENROLL_CODE_TTL_SECS` | 300 | Life of an enrollment code. |
| `LAB_UTC_OFFSET_HOURS` | 8 | The room's clock, used for after-hours checks and dates. |
| `SHEETS_ENABLED` | false | Optional mirror of the trail into Google Sheets. |

Values containing spaces must be quoted. Anything after an unquoted space is silently
dropped by the parser, which is a good way to lose an afternoon.

## Security notes

- Passwords are hashed with argon2. The policy is 8 to 16 characters with an upper, a
  lower and a symbol, enforced in the backend and mirrored in the UI.
- Readers authenticate with a device id and secret pair, never a user token, so a stolen
  reader cannot impersonate a person. Secrets can be rotated from the web app.
- Fingerprint templates never leave the sensor. The server only ever sees an opaque token.
- Reader events cannot be edited or deleted through the API by anyone. A wrong record is
  corrected by adding the right one.
- `.env` files, the database, and uploaded photos are excluded from version control. Every
  credential in the repository is a placeholder.

## Status

Backend, frontend and firmware are complete and build clean, with the backend test suite
and `svelte-check` passing. The firmware compiles for the ESP32 but has not been run on
assembled hardware yet, so treat pin assignments and timings as correct-but-unproven until
you have flashed a board. The SMS and Google Sheets paths were verified end to end against
local stand-ins rather than a real modem or a real Google account.

## License

Apache License 2.0. See [LICENSE](LICENSE).
