use std::sync::{Arc, Mutex, RwLock};
use log::warn;
use sdl2::audio::AudioCallback;

pub struct SquareWave {
    sound_timer: Arc<Mutex<u8>>,
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl SquareWave {
    pub fn new(sound_timer: Arc<Mutex<u8>>,
               phase_inc: f32,
               volume: f32)->SquareWave {
        SquareWave {
            sound_timer,
            phase: 0f32,
            phase_inc,
            volume,
        }
    }
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let sound_timer = self.sound_timer.lock().expect("Could not lock to play audio");
        let sound_timer = sound_timer.clone();
        // log::info!("Processing audio buffer length {}",out.len());
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if sound_timer > 0 {
                if self.phase <= 0.5 {
                    self.volume
                } else {
                    -self.volume
                }
            } else {
                0f32
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}