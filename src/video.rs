//! Video decoder: spawns ffmpeg and reads raw frames into a bounded buffer.
//! Output is RGBA. Optionally cuts out a green screen (chroma key).

use anyhow::{Context, Result};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub src_width: u32,
    pub src_height: u32,
    pub fps: f32,
}

/// Use the sidecar ffmpeg if it was downloaded, otherwise the system one from PATH.
pub fn resolve_ffmpeg() -> PathBuf {
    use ffmpeg_sidecar::paths::ffmpeg_path;
    let sidecar = ffmpeg_path();
    if sidecar.exists() {
        sidecar
    } else {
        PathBuf::from("ffmpeg")
    }
}

/// Is a working ffmpeg available (sidecar or system)?
pub fn ffmpeg_available() -> bool {
    std::process::Command::new(resolve_ffmpeg())
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

pub struct VideoPlayer {
    /// Output frame size (RGBA)
    pub width: u32,
    pub height: u32,
    pub info: VideoInfo,
    buffer: Arc<FrameBuffer>,
    stop: Arc<AtomicBool>,
}

struct FrameBuffer {
    frames: Mutex<VecDeque<Vec<u8>>>,
    cond: Condvar,
    cap: usize,
    closed: AtomicBool,
}

impl FrameBuffer {
    /// Push a frame. Blocks while the buffer is full, which throttles ffmpeg
    /// through pipe backpressure. This guarantees sequential frames with no drops.
    fn push(&self, frame: Vec<u8>) {
        let mut q = self.frames.lock().unwrap();
        while q.len() >= self.cap && !self.closed.load(Ordering::Relaxed) {
            q = self.cond.wait(q).unwrap();
        }
        if self.closed.load(Ordering::Relaxed) {
            return;
        }
        q.push_back(frame);
        self.cond.notify_all();
    }

    fn pop(&self, wait: Duration) -> Option<Vec<u8>> {
        let mut q = self.frames.lock().unwrap();
        if q.is_empty() {
            let (guard, _timeout) = self
                .cond
                .wait_timeout_while(q, wait, |q| q.is_empty() && !self.closed.load(Ordering::Relaxed))
                .unwrap();
            q = guard;
        }
        let frame = q.pop_front();
        if frame.is_some() {
            // Wake the (possibly blocked) producer
            self.cond.notify_all();
        }
        frame
    }
}

impl VideoPlayer {
    /// Open a video. `out_width`/`out_height` are the output frame size.
    /// `chroma_key` cuts the green background out into transparency.
    pub fn open(
        video_path: &Path,
        out_width: u32,
        out_height: u32,
        chroma_key: bool,
    ) -> Result<Self> {
        let info = probe(video_path)
            .with_context(|| format!("ffmpeg could not read {:?}", video_path))?;
        let player = Self {
            width: out_width,
            height: out_height,
            info,
            buffer: Arc::new(FrameBuffer {
                frames: Mutex::new(VecDeque::new()),
                cond: Condvar::new(),
                cap: 12,
                closed: AtomicBool::new(false),
            }),
            stop: Arc::new(AtomicBool::new(false)),
        };
        player.spawn_decoder(video_path, chroma_key);
        Ok(player)
    }

    fn spawn_decoder(&self, video_path: &Path, chroma_key: bool) {
        let ffmpeg = resolve_ffmpeg();
        let path = video_path.to_path_buf();
        let w = self.width;
        let h = self.height;
        let buffer = Arc::clone(&self.buffer);
        let stop = Arc::clone(&self.stop);

        std::thread::spawn(move || {
            // -stream_loop -1: loop forever, -an: no audio needed
            let mut cmd = std::process::Command::new(&ffmpeg);
            cmd.args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-stream_loop",
                "-1",
                "-i",
            ])
            .arg(&path)
            .args([
                "-an",
                "-vf",
                &format!("scale={w}:{h}"),
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "pipe:1",
            ]);
            cmd.stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("ffmpeg failed to start: {e}");
                    return;
                }
            };

            let Some(stdout) = child.stdout.take() else {
                return;
            };

            let frame_size = (w * h * 3) as usize;
            let out_size = (w * h * 4) as usize;
            let mut reader = std::io::BufReader::with_capacity(frame_size * 4, stdout);
            let mut rgba = vec![0u8; out_size];

            loop {
                use std::io::Read;
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                let mut rgb = vec![0u8; frame_size];
                match reader.read_exact(&mut rgb) {
                    Ok(()) => {
                        if chroma_key {
                            chroma_to_rgba(&rgb, &mut rgba);
                        } else {
                            rgb_to_rgba(&rgb, &mut rgba);
                        }
                        buffer.push(rgba.clone());
                    }
                    Err(_) => break,
                }
            }
            let _ = child.kill();
            let _ = child.wait();
        });
    }

    /// Duration of a single video frame
    pub fn frame_interval(&self) -> Duration {
        let fps = if self.info.fps > 1.0 { self.info.fps } else { 30.0 };
        Duration::from_secs_f64(1.0 / fps as f64)
    }

    /// Take the next frame (waits up to `wait` if the buffer is empty)
    pub fn next_frame(&self, wait: Duration) -> Option<Vec<u8>> {
        self.buffer.pop(wait)
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.buffer.closed.store(true, Ordering::Relaxed);
        self.buffer.cond.notify_all();
    }
}

/// RGB24 -> RGBA (alpha 255)
fn rgb_to_rgba(rgb: &[u8], out: &mut [u8]) {
    for (dst, px) in out.chunks_exact_mut(4).zip(rgb.chunks_exact(3)) {
        dst[0] = px[0];
        dst[1] = px[1];
        dst[2] = px[2];
        dst[3] = 255;
    }
}

/// Chroma key: green background -> transparency, with a light despill on edges.
/// The meme silhouette is black, so a simple "greenness" threshold is enough.
fn chroma_to_rgba(rgb: &[u8], out: &mut [u8]) {
    for (dst, px) in out.chunks_exact_mut(4).zip(rgb.chunks_exact(3)) {
        let (r, g, b) = (px[0] as i32, px[1] as i32, px[2] as i32);
        let greenness = g - r.max(b);
        if greenness > 70 {
            // pure background
            dst[0] = 0;
            dst[1] = 0;
            dst[2] = 0;
            dst[3] = 0;
        } else if greenness > 20 {
            // semi-transparent edge + remove the green tint
            let alpha = (255 - (greenness - 20) * 255 / 50).clamp(0, 255) as u8;
            let m = r.max(b).min(255) as u8;
            dst[0] = r as u8;
            dst[1] = m;
            dst[2] = b as u8;
            dst[3] = alpha;
        } else {
            dst[0] = r as u8;
            dst[1] = g as u8;
            dst[2] = b as u8;
            dst[3] = 255;
        }
    }
}

/// Read width, height and fps. There is no ffprobe in the sidecar package,
/// so we parse the stderr of a plain ffmpeg run (-i with no output file).
fn probe(path: &Path) -> Result<VideoInfo> {
    let out = std::process::Command::new(resolve_ffmpeg())
        .arg("-hide_banner")
        .arg("-i")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run ffmpeg")?;

    let s = String::from_utf8_lossy(&out.stderr);
    let video_line = s
        .lines()
        .find(|l| l.contains("Video:"))
        .context("no video stream in the file")?;

    let src_width = parse_dims(video_line)
        .map(|(w, _)| w)
        .context("could not find width")?;
    let src_height = parse_dims(video_line)
        .map(|(_, h)| h)
        .context("could not find height")?;
    let fps = parse_fps(video_line).unwrap_or(30.0);

    Ok(VideoInfo {
        src_width,
        src_height,
        fps,
    })
}

/// Find "WxH" (e.g. "720x1280") in a string
fn parse_dims(line: &str) -> Option<(u32, u32)> {
    for token in line.split(|c: char| c == ',' || c == ' ' || c == '[') {
        let t = token.trim();
        if let Some((w, h)) = t.split_once('x') {
            if let (Ok(w), Ok(h)) = (w.parse::<u32>(), h.parse::<u32>()) {
                if w >= 16 && h >= 16 {
                    return Some((w, h));
                }
            }
        }
    }
    None
}

/// Find "... fps" (e.g. "43.78 fps") in a string
fn parse_fps(line: &str) -> Option<f32> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    for (i, t) in tokens.iter().enumerate() {
        if t.contains("fps") && i > 0 {
            if let Ok(f) = tokens[i - 1].trim_end_matches(',').parse::<f32>() {
                if f > 1.0 {
                    return Some(f);
                }
            }
        }
    }
    None
}
