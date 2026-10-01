//! Sound: a dedicated thread owns the audio device and receives commands over a channel.
//! rodio 0.22: MixerDeviceSink + Player (OutputStream/Sink were removed).

use rand::RngExt;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};

pub enum SoundCmd {
    Play { path: PathBuf, volume: f32 },
    /// Reserved: volume control from the tray (v0.2)
    #[allow(dead_code)]
    SetVolume(f32),
}

pub struct SoundPlayer {
    tx: Sender<SoundCmd>,
    pub sounds: Vec<PathBuf>,
}

impl SoundPlayer {
    /// Scan the sounds/ folder and start the audio thread.
    pub fn new(sounds_dir: &std::path::Path) -> Self {
        let sounds = scan_sounds(sounds_dir);

        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || audio_thread(rx));

        Self { tx, sounds }
    }

    /// Re-scan the sounds folder, so files added while running are picked up.
    pub fn refresh(&mut self, sounds_dir: &std::path::Path) {
        self.sounds = scan_sounds(sounds_dir);
    }

    /// Play a random sound. Returns an approximate duration in seconds.
    pub fn play_random(&self, volume: f32) -> f32 {
        let mut rng = rand::rng();
        if self.sounds.is_empty() {
            return 0.0;
        }
        let path = self.sounds[rng.random_range(0..self.sounds.len())].clone();
        let _ = self.tx.send(SoundCmd::Play { path, volume });
        // Rough estimate: 2–5 s, precision is not critical
        rng.random_range(2.0..5.0)
    }

    /// Play a random landing sound (file name starts with `hit_`).
    /// Returns false if there are none.
    pub fn play_hit(&self, volume: f32) -> bool {
        let hits: Vec<&PathBuf> = self
            .sounds
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_ascii_lowercase().starts_with("hit_"))
                    .unwrap_or(false)
            })
            .collect();
        if hits.is_empty() {
            return false;
        }
        let mut rng = rand::rng();
        let path = hits[rng.random_range(0..hits.len())].clone();
        let _ = self.tx.send(SoundCmd::Play { path: path.clone(), volume });
        true
    }

    /// Reserved: volume control from the tray (v0.2)
    #[allow(dead_code)]
    pub fn set_volume(&self, volume: f32) {
        let _ = self.tx.send(SoundCmd::SetVolume(volume));
    }

    pub fn has_sounds(&self) -> bool {
        !self.sounds.is_empty()
    }
}

/// Collect supported sound files from a folder.
fn scan_sounds(dir: &std::path::Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    matches!(
                        p.extension().and_then(|e| e.to_str()),
                        Some("mp3") | Some("wav") | Some("ogg") | Some("flac")
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn audio_thread(rx: Receiver<SoundCmd>) {
    let sink = match rodio::stream::DeviceSinkBuilder::open_default_sink() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not open audio device ({e}), sound disabled");
            // Still drain commands so the sender never blocks
            for _ in rx {}
            return;
        }
    };

    let player = rodio::Player::connect_new(sink.mixer());
    player.set_volume(0.8);

    for cmd in rx {
        match cmd {
            SoundCmd::Play { path, volume } => match std::fs::File::open(&path) {
                Ok(file) => match rodio::Decoder::try_from(file) {
                    Ok(source) => {
                        player.clear();
                        player.set_volume(volume);
                        player.append(source);
                    }
                    Err(e) => eprintln!("Failed to decode {:?}: {e}", path),
                },
                Err(e) => eprintln!("Failed to open {:?}: {e}", path),
            },
            SoundCmd::SetVolume(v) => {
                player.set_volume(v);
            }
        }
    }
}
