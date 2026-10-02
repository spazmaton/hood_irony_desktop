<div align="center">
  <img src="assets/pet.gif" width="150" alt="Hood Irony walking across the desktop">
  <h1>HOOD IRONY DESKTOP PET</h1>
  <p><b>A lightweight animated companion for Windows.</b></p>
  <p>
    <img alt="platform" src="https://img.shields.io/badge/platform-Windows-0078D6?style=flat-square">
    <img alt="rust" src="https://img.shields.io/badge/rust-2021-000000?style=flat-square&logo=rust">
    <img alt="status" src="https://img.shields.io/badge/version-0.2-4ade80?style=flat-square">
    <img alt="license" src="https://img.shields.io/badge/license-MIT-blue?style=flat-square">
  </p>
</div>

---

Hood Irony is a small Windows desktop pet built from a looping video. It walks
across the screen, occasionally plays meme sounds, and reacts to dragging: pick
it up, toss it, and watch it bounce off the screen edges.

Keep the green background for the original meme look, or remove it in Settings.

## Features

- Walks and pauses from time to time.
- Follows the cursor when dragged, then falls and bounces when thrown.
- Plays random sounds and separate landing sounds.
- Lets you adjust its size, speed, volume, and other settings.
- Can be controlled from the system tray menu.

The pet currently runs in a single floating window. It does not walk on top of
other windows or the taskbar, and multi-monitor behavior has not been
implemented. It is a lightweight desktop companion rather than a full Shimeji.

## Run from source

```powershell
cargo run --release
```

Requires Rust with the MSVC toolchain. On first run, `ffmpeg-sidecar` downloads
FFmpeg if a system installation is not available.

## Build

```powershell
cargo build --release
```

The executable is written to `target/release/hood-irony-desktop.exe`.

## Assets

| Path | Contents |
|---|---|
| `assets/video/pet.mp4` | Pet video; loops automatically |
| `assets/sounds/` | MP3, WAV, OGG, or FLAC sounds; files named `hit_*` are used for landings |
| `assets/icon.png` | System tray icon (64×64 recommended) |

See [docs/ASSETS.md](docs/ASSETS.md) for details.

## Configuration

On first run, `config.toml` is created next to the executable (in the project
root for debug builds). Settings are also available in the app window. See
[docs/CONFIG.md](docs/CONFIG.md) for details.

## Tray menu

- **Pause** — pauses the pet.
- **Settings** — adjust size, speed, sound, and appearance.
- **Quit** — close the app.

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — application structure and behavior
- [docs/ROADMAP.md](docs/ROADMAP.md) — development roadmap
- [docs/ASSETS.md](docs/ASSETS.md) — video and sound requirements
- [docs/CONFIG.md](docs/CONFIG.md) — `config.toml` options
- [docs/TRICKS.md](docs/TRICKS.md) — Windows and egui notes

## Roadmap

- **v0.3** — walk on top of application windows and the taskbar.
- Later — click reactions, alternate videos, and launch on Windows startup.

## Security note

This build is unsigned, so Windows or antivirus software may show a warning.
Do not disable security protections or add broad exclusions just to run it.
Build from source or use a binary from a source you trust. See
[docs/TRICKS.md](docs/TRICKS.md) for more Windows notes.

## License

[MIT](LICENSE) — see the license file for terms.
