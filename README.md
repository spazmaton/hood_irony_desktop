# Hood Irony Desktop 🚶‍♂️🟩

A tiny Windows desktop pet: the black silhouette from the **hood irony** meme
(a green-screen video) walks along the bottom of your screen and randomly plays
meme sounds.

> The pet literally *is* a vertical video walking around your desktop.
> The green screen is a feature, not a bug.

## Features

- 🔁 Looped mp4 playback in an always-on-top, borderless overlay window
- 🫥 No taskbar entry, no Alt+Tab entry, never steals focus
- 🚶 Walks along the bottom edge, turns around at screen edges (mirrored video)
- 🔊 Random meme sounds from `assets/sounds/` (interval configurable)
- 🖱️ Drag with the mouse; it falls with gravity when released
- 📟 Tray icon: Pause / Settings / Quit
- ⚙️ Settings window with a live preview
- ⚙️ `config.toml`

## Run from source

```powershell
cargo run --release
```

Requires Rust with the MSVC toolchain. FFmpeg is pulled in automatically by
`ffmpeg_sidecar`; if it is already installed on the system, that one is used.

## Build

```powershell
cargo build --release
```

Binary: `target/release/hood-irony-desktop.exe`

## Assets

| Folder | What to put there |
|---|---|
| `assets/video/` | `pet.mp4` — the pet video (loops automatically) |
| `assets/sounds/` | `*.mp3` / `*.wav` — sounds for random playback |
| `assets/icon.png` | tray icon (64×64) |

See [docs/ASSETS.md](docs/ASSETS.md).

## Configuration

`config.toml` is created next to the exe on first run (in the project root in
debug builds). It can also be edited live in the Settings window.
See [docs/CONFIG.md](docs/CONFIG.md).

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — architecture, crates, state machine
- [docs/ROADMAP.md](docs/ROADMAP.md) — development stages and status
- [docs/ASSETS.md](docs/ASSETS.md) — video and sound requirements
- [docs/CONFIG.md](docs/CONFIG.md) — config options
- [docs/TRICKS.md](docs/TRICKS.md) — Windows/egui pitfalls and fixes

## Ideas for later

- [ ] Chroma-key (green screen) cutout option — already implemented, off by default
- [ ] Volume control from the tray
- [ ] Multi-monitor support
- [ ] Remember position between runs
