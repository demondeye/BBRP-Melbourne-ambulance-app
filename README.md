# Victoria Ambulance — Rust desktop app

A native Windows version of `victoria-ambulance`, rewritten in Rust with a fully
native UI (egui). No browser, no Node, no web code: one `VictoriaAmbulance.exe`.

## Run it

Double-click `VictoriaAmbulance.exe`. It opens maximised. **F11** toggles full screen.

Same flow as the web version: boot screen (20 s, with the startup chime) →
login / create local account → desktop with menu bar, dock, launcher and windows.

## What carried over

- Boot, login (local account, forgot-password reset), restart / shut down / log out
- Desktop: menu bar, dock (with right-click menu), launcher with search, desktop
  right-click menu, Cut / Copy / Paste / Select All menu on text boxes, toasts,
  "Are you sure?" dialogs
- Windows: drag, resize, minimise, maximise (double-click title bar), stacking,
  sizes and positions remembered between runs; small screens fill the desktop
- Apps: Time Clock (months, month picker, edit / delete / clear shifts),
  Field Guide (all pages, choices, MICA question, progress, confetti),
  Settings (wallpapers + your own picture, Stream Deck, Account, About),
  Patient Records, Notes and About placeholders
- **Stream Deck control API** on `http://localhost:47913` with the same addresses
  and replies. No API key: it only answers this PC, refuses web pages, and
  controls whoever is signed in. The app has to be open for the keys to work.

## Software updates

The app updates itself like a system update. It checks the Releases of this
repo, `github.com/demondeye/BBRP-Melbourne-ambulance-app` (set in
`src/update.rs`), when it starts and every 3 hours. When a newer version is published:

- a popup says **New update is ready to install** — *Install now* or *Later*
- **Settings → Software Update** shows it (with a red badge), with *Install now*
  and *Check for updates*; the system menu shows *Update ready to install…*

Installing downloads the new `VictoriaAmbulance.exe`, checks it against GitHub's
checksum, swaps it in, shows an "Installing update" screen and restarts the app.
You log in again; the Time Clock and everything else is kept.

The exe must sit in a folder you can write to (Desktop, Documents, etc. — not
Program Files) for it to update itself.

### Publishing an update

1. Make your changes.
2. Raise `version` in `Cargo.toml` (e.g. `0.2.0` → `0.3.0`).
3. Write what's new in `RELEASE_NOTES.md` (shown in the popup).
4. Run `powershell -ExecutionPolicy Bypass -File release.ps1`

`release.ps1` builds the exe and publishes it as a release tagged `v0.3.0` (needs
the GitHub CLI, signed in once with `gh auth login`). Commit and push your code
first, so the release matches it.

You can also publish by hand: on the repo, *Releases → Draft a new
release*, tag `v0.3.0`, title `Victoria Ambulance 0.3.0`, paste the notes, attach
`VictoriaAmbulance.exe`, *Publish release*.

The repo has to stay public so the app can read it without a password.

## Where data is saved

`%APPDATA%\Victoria Ambulance\` — account, per-user settings, time sheets, window
layout and wallpaper pictures. (The web version kept these in the browser, so
nothing moves across automatically: create the account again.)

## Build it yourself

Needs Rust (https://rustup.rs).

```
cargo build --release
```

The exe lands in `target\release\VictoriaAmbulance.exe`.

From Linux, cross-compile with `rustup target add x86_64-pc-windows-gnu`, install
`mingw-w64`, then `cargo build --release --target x86_64-pc-windows-gnu`.

For testing only: `VA_FAST=1` shortens the boot and login waits, `VA_WINDOWED=1`
starts in a window, `VA_DATA_DIR=<folder>` keeps a separate set of data, `VA_UPDATE_URL=<url>` reads updates
from a different release feed.

## Layout

```
src/main.rs            window setup
src/app.rs             boot → login → desktop, restart / shut down
src/core.rs            state shared with the Stream Deck server
src/relay.rs           Stream Deck control API (localhost:47913)
src/update.rs          software updates (GitHub Releases)
src/auth.rs            local account
src/settings.rs        per-user settings, wallpapers
src/timeclock.rs       clock in / out, shifts
src/fieldguide/        Field Guide content (data.rs) and logic (store.rs)
src/ui/                desktop, window manager, theme, icons, wallpaper
src/ui/apps/           Time Clock, Field Guide, Settings, placeholders
assets/                Outfit + Syne fonts, Twemoji pictures, app icon
```

Field Guide text lives in `src/fieldguide/data.rs`, as before.

Emoji pictures are Twemoji (CC-BY 4.0, © Twitter / jdecked). Fonts are Outfit and
Syne (SIL Open Font License).
