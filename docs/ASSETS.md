# Assets

## Pet video

- Path: `assets/video/pet.mp4`
- Format: anything ffmpeg can read (mp4/h264 is the reference)
- Orientation: vertical (9:16) — the pet window follows the video aspect
- Audio track: ignored (muted), can be absent
- Looping: built in (`-stream_loop -1`); ideally the first and last frames
  match so the seam is invisible
- Current video: `Hood irony walking - Timmy Turner (720p, h264).mp4`
  — silhouette on a green screen, 720×1280, 44 fps, 13.5 s, 593 frames.

## README banner

- Path: `assets/pet.gif`
- A short loop cut from the video (`ffmpeg ... palettegen/paletteuse`),
  shown at the top of the README. Regenerate it if the video changes.

## Sounds

- Path: `assets/sounds/`
- Format: `*.mp3` or `*.wav`, any names (ASCII without spaces is safer)
- How many: 3–10 short clips (1–5 s) from the hood irony meme
- Selection: random file at a random interval (default 30–120 s)
- Landing: files whose name starts with `hit_` (e.g. `hit_slap.mp3`) play
  when the pet hits the ground after a throw/drag

## Tray icon

- Path: `assets/icon.png`
- A small 64×64 square; the silhouette on green works great
