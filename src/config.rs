use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Width of the video on screen in px (height follows the video aspect)
    pub pet_width: f32,
    /// Walk speed in px/s
    pub walk_speed: f32,
    /// Interval between random sounds in seconds
    pub sound_interval_min: f32,
    pub sound_interval_max: f32,
    /// Sound volume 0.0..=1.0
    pub volume: f32,
    /// Output device name; empty = system default
    pub audio_device: String,
    /// Video playback speed (1.0 = original, <1 slower)
    pub video_speed: f32,
    /// Derive playback speed from walk_speed so the step matches the movement
    pub animation_sync: bool,
    /// Walk speed (px/s) at which playback is 1x; used by animation_sync
    pub sync_walk_at_1x: f32,
    /// Cut out the green screen (shimeji mode). false = green screen, like the meme
    pub chroma_key: bool,
    /// Keep animating while idle (true) or only while walking (false)
    pub animate_when_idle: bool,
    /// Play the video's own audio track
    pub video_audio: bool,
    /// Freeze while a sound is playing
    pub stop_on_sound: bool,
    /// Pause (kept in sync with the tray checkbox)
    pub paused: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pet_width: 180.0,
            walk_speed: 45.0,
            sound_interval_min: 30.0,
            sound_interval_max: 120.0,
            volume: 0.8,
            audio_device: String::new(),
            video_speed: 0.5,
            animation_sync: false,
            sync_walk_at_1x: 45.0,
            chroma_key: false,
            animate_when_idle: false,
            video_audio: false,
            stop_on_sound: true,
            paused: false,
        }
    }
}

/// Application data root: the project folder in debug builds,
/// the folder next to the exe in release builds.
pub fn app_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    } else {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

pub fn config_path() -> PathBuf {
    app_dir().join("config.toml")
}

pub fn assets_dir() -> PathBuf {
    app_dir().join("assets")
}

impl Config {
    /// Fix out-of-range values so a hand-edited config can never crash the app.
    pub fn sanitize(&mut self) {
        if !self.sound_interval_min.is_finite() {
            self.sound_interval_min = 30.0;
        }
        self.sound_interval_min = self.sound_interval_min.clamp(0.1, 599.5);
        if !self.sound_interval_max.is_finite() {
            self.sound_interval_max = 120.0;
        }
        self.sound_interval_max = self.sound_interval_max.clamp(0.6, 600.0);
        self.sound_interval_max = self.sound_interval_max.max(self.sound_interval_min + 0.5);
        if !self.pet_width.is_finite() {
            self.pet_width = 180.0;
        }
        self.pet_width = self.pet_width.clamp(24.0, 2000.0);
        if !self.walk_speed.is_finite() {
            self.walk_speed = 45.0;
        }
        self.walk_speed = self.walk_speed.clamp(1.0, 2000.0);
        if !self.video_speed.is_finite() {
            self.video_speed = 0.5;
        }
        self.video_speed = self.video_speed.clamp(0.05, 4.0);
        if !self.sync_walk_at_1x.is_finite() {
            self.sync_walk_at_1x = 45.0;
        }
        self.sync_walk_at_1x = self.sync_walk_at_1x.clamp(5.0, 300.0);
        if !self.volume.is_finite() {
            self.volume = 0.8;
        }
        self.volume = self.volume.clamp(0.0, 1.0);
    }

    pub fn load() -> Self {
        let path = config_path();
        match fs::read_to_string(&path) {
            Ok(s) => match toml::from_str::<Config>(&s) {
                Ok(mut cfg) => {
                    cfg.sanitize();
                    cfg
                }
                Err(e) => {
                    eprintln!("Invalid config.toml ({e}), using defaults");
                    Self::default()
                }
            },
            Err(_) => {
                let cfg = Self::default();
                let _ = cfg.save();
                cfg
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let s = toml::to_string_pretty(self)?;
        fs::write(config_path(), s)?;
        Ok(())
    }
}
