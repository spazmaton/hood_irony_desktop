//! Pet behavior state machine: Idle / Walk / Sound / Dragged / Drop

use crate::config::Config;
use rand::RngExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    Walk,
    Sound,
    Dragged,
    /// Falling / thrown after being released
    Drop,
}

/// Events produced by one `update` step.
#[derive(Debug, Default, Clone, Copy)]
pub struct PetUpdate {
    /// A random sound should be played
    pub random_sound: bool,
    /// The pet just hit the ground (for a landing sound)
    pub landed: bool,
}

pub struct Pet {
    /// Window top-left position in logical points
    pub pos: egui::Vec2,
    /// Horizontal velocity (px/s)
    pub vx: f32,
    /// Vertical velocity (px/s)
    pub vy: f32,
    pub state: State,
    /// Timer until next state change
    pub timer: f32,
    /// Timer until the next random sound
    pub sound_timer: f32,
    /// Screen size in logical points
    pub screen: egui::Vec2,
    /// Pet window size in logical points
    pub size: egui::Vec2,
    /// Offset of the cursor inside the window when the drag started
    grab: egui::Vec2,
    /// Smoothed drag velocity, used as throw inertia
    drag_vel: egui::Vec2,
    /// Current facing direction (1 = right, -1 = left)
    facing_dir: f32,
    /// State and timer to resume after a sound temporarily freezes movement
    sound_return_state: State,
    sound_return_timer: f32,
}

const GRAVITY: f32 = 2200.0;
const BOUNCE: f32 = 0.45;
const AIR_DRAG: f32 = 0.6;
const MAX_THROW: f32 = 2600.0;

impl Pet {
    pub fn new(size: egui::Vec2, screen: egui::Vec2, cfg: &Config) -> Self {
        let mut rng = rand::rng();
        let dir: f32 = if rng.random_bool(0.5) { 1.0 } else { -1.0 };
        Self {
            pos: egui::Vec2::new(screen.x * 0.5 - size.x * 0.5, screen.y - size.y),
            vx: 40.0 * dir,
            vy: 0.0,
            state: State::Idle,
            timer: 1.0,
            sound_timer: rng.random_range(cfg.sound_interval_min..cfg.sound_interval_max),
            screen,
            size,
            grab: egui::Vec2::ZERO,
            drag_vel: egui::Vec2::ZERO,
            facing_dir: dir,
            sound_return_state: State::Idle,
            sound_return_timer: 1.0,
        }
    }

    pub fn walk_speed(&self, cfg: &Config) -> f32 {
        cfg.walk_speed
    }

    /// Start dragging. `local` is the cursor position inside the window.
    pub fn begin_drag(&mut self, local: egui::Vec2) {
        self.state = State::Dragged;
        self.grab = local;
        self.drag_vel = egui::Vec2::ZERO;
        self.vx = 0.0;
        self.vy = 0.0;
    }

    /// Update position while dragging and estimate the throw velocity.
    ///
    /// egui reports the cursor position *inside the window*, not on the screen.
    /// With the window's top-left at `pos`, the global cursor is `pos + local`.
    /// Keeping the grab offset constant: `pos = pos + local - grab`.
    pub fn drag_update(&mut self, local: egui::Vec2, dt: f32) {
        if self.state != State::Dragged {
            return;
        }
        let new_pos = self.pos + local - self.grab;
        if dt > 1e-4 {
            let instant = (new_pos - self.pos) / dt;
            // Exponential smoothing so a single jittery frame does not dominate
            self.drag_vel += (instant - self.drag_vel) * 0.35;
        }
        self.pos = new_pos;
        if self.drag_vel.x.abs() > 30.0 {
            self.facing_dir = self.drag_vel.x.signum();
        }
    }

    /// Release the pet: it keeps the drag velocity as throw inertia.
    pub fn release(&mut self) {
        if self.state == State::Dragged {
            self.state = State::Drop;
            self.vx = self.drag_vel.x.clamp(-MAX_THROW, MAX_THROW);
            self.vy = self.drag_vel.y.clamp(-MAX_THROW, MAX_THROW);
        }
    }

    fn floor_y(&self) -> f32 {
        self.screen.y - self.size.y
    }

    pub fn floor_y_pub(&self) -> f32 {
        self.floor_y()
    }

    pub fn play_sound(&mut self, duration: f32, cfg: &Config) {
        if cfg.stop_on_sound {
            if self.state != State::Sound {
                self.sound_return_state = self.state;
                self.sound_return_timer = self.timer;
            }
            self.state = State::Sound;
            self.timer = duration.max(0.5);
        }
    }

    /// Advance the logic by `dt` seconds.
    pub fn update(&mut self, dt: f32, cfg: &Config) -> PetUpdate {
        let mut out = PetUpdate::default();
        if cfg.paused {
            return out;
        }

        // The sound timer ticks unless we are dragging
        if self.state != State::Dragged {
            self.sound_timer -= dt;
            if self.sound_timer <= 0.0 {
                let mut rng = rand::rng();
                // Guard against an empty range (min == max) which would panic
                let lo = cfg.sound_interval_min.max(0.1);
                let hi = cfg.sound_interval_max.max(lo + 0.1);
                self.sound_timer = rng.random_range(lo..hi);
                out.random_sound = true;
            }
        }

        match self.state {
            State::Idle => {
                self.timer -= dt;
                if self.timer <= 0.0 {
                    let mut rng = rand::rng();
                    self.state = State::Walk;
                    self.vx = self.walk_speed(cfg) * if rng.random_bool(0.5) { 1.0 } else { -1.0 };
                }
            }
            State::Walk => {
                self.pos.x += self.vx * dt;
                self.facing_dir = if self.vx >= 0.0 { 1.0 } else { -1.0 };
                // Turn around at the screen edges
                if self.pos.x <= 0.0 {
                    self.pos.x = 0.0;
                    self.vx = self.vx.abs();
                } else if self.pos.x + self.size.x >= self.screen.x {
                    self.pos.x = self.screen.x - self.size.x;
                    self.vx = -self.vx.abs();
                }
                // Occasional random pause
                let mut rng = rand::rng();
                if rng.random_bool(dt as f64 * 0.05) {
                    self.state = State::Idle;
                    self.timer = rng.random_range(1.0..4.0);
                }
            }
            State::Sound => {
                self.timer -= dt;
                if self.timer <= 0.0 {
                    self.state = self.sound_return_state;
                    self.timer = self.sound_return_timer;
                }
            }
            State::Dragged => {
                // Position is driven by the cursor
            }
            State::Drop => {
                self.vy += GRAVITY * dt;
                self.pos.x += self.vx * dt;
                self.pos.y += self.vy * dt;
                if self.vx.abs() > 30.0 {
                    self.facing_dir = self.vx.signum();
                }

                // Ceiling bounce (top of the screen)
                if self.pos.y <= 0.0 {
                    self.pos.y = 0.0;
                    if self.vy < 0.0 {
                        self.vy = -self.vy * BOUNCE;
                    }
                }

                // Side walls bounce
                if self.pos.x <= 0.0 {
                    self.pos.x = 0.0;
                    self.vx = -self.vx * BOUNCE;
                } else if self.pos.x + self.size.x >= self.screen.x {
                    self.pos.x = self.screen.x - self.size.x;
                    self.vx = -self.vx * BOUNCE;
                }

                // Floor
                if self.pos.y >= self.floor_y() {
                    self.pos.y = self.floor_y();
                    out.landed = true;
                    if self.vy.abs() > 380.0 {
                        self.vy = -self.vy * BOUNCE; // bounce
                    } else {
                        self.vy = 0.0;
                        self.vx = 0.0;
                        self.state = State::Idle;
                        self.timer = 1.0;
                    }
                }

                self.vx *= 1.0 - AIR_DRAG * dt;
            }
        }

        // Safety net: the pet must never leave the screen. If the window went
        // off-screen its updates would stop (egui skips hidden windows) and it
        // would never come back, so clamp it into the visible area.
        let max_x = (self.screen.x - self.size.x).max(0.0);
        let max_y = self.floor_y().max(0.0);
        self.pos.x = self.pos.x.clamp(0.0, max_x);
        self.pos.y = self.pos.y.clamp(0.0, max_y);

        out
    }

    /// Whether the animation should play right now.
    pub fn animating(&self) -> bool {
        matches!(self.state, State::Walk | State::Drop)
    }

    /// Facing direction: 1 = right, -1 = left
    pub fn facing(&self) -> f32 {
        self.facing_dir
    }
}
