# Assets

## Pet video

- Path: `assets/video/pet.mp4`
- Format: anything FFmpeg can read; MP4/H.264 is a good choice.
- Orientation: portrait. The pet window follows the video's aspect ratio.
- The video's audio track is not played, so it can be omitted.
- The video loops using `-stream_loop -1`. For a seamless loop, make the first
  and last frames similar.
- The reference video, `Hood irony walking - Timmy Turner (720p, h264).mp4`,
  shows a silhouette on a green screen at 720×1280, 44 fps, and 13.5 seconds.

## README banner

- Path: `assets/pet.gif`
- A short video loop shown at the top of the README. Regenerate it with FFmpeg's
  `palettegen`/`paletteuse` filters if the video changes.

## Sounds

- Path: `assets/sounds/`
- Formats: MP3, WAV, OGG, or FLAC. File extensions are case-insensitive.
- Random sounds are selected at the interval set in Settings (30–120 seconds
  by default).
- Files whose names start with `hit_` (for example, `hit_slap.mp3`) are used
  when the pet lands after a throw.

## Tray icon

- Path: `assets/icon.png`
- A square image around 64×64 pixels is recommended.
