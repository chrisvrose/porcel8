use std::sync::{Arc, Mutex};
use sdl2::audio::AudioQueue;
use crate::util::EmulatorResult;

pub struct SdlAudioAdapter {
    sound_timer: Arc<Mutex<u8>>,
    phase_inc: f32,
    phase: f32,
    volume: f32,
    audio_queue: AudioQueue<f32>,
    buf: Vec<f32>,
}

/// An Audio adapter using `AudioQueue`.
impl SdlAudioAdapter {
    pub const SAMPLING_FREQ:i32 = 15360;
    pub const SAMPLES_PER_FRAME: usize = (Self::SAMPLING_FREQ as usize / 60) * 2;
    pub fn new(sound_timer: Arc<Mutex<u8>>,
               freq: f32,
               volume: f32,
               audio_queue: AudioQueue<f32>) -> SdlAudioAdapter {
        audio_queue.resume();
        SdlAudioAdapter {
            sound_timer,
            buf: vec![0f32; Self::SAMPLES_PER_FRAME],
            phase: 0f32,
            phase_inc: freq/Self::SAMPLING_FREQ as f32,
            volume,
            audio_queue,
        }
    }
    pub fn process_push_audio(&mut self) -> EmulatorResult<()> {
        // fill the audio vector.
        let sound_timer = {
            let sound_timer = self.sound_timer.lock().expect("Could not lock to play audio");
            sound_timer.clone()
        };
        if sound_timer>0 && self.audio_queue.size() < Self::SAMPLING_FREQ as u32 {
            self.fill_audio();
            self.audio_queue.queue_audio(&self.buf)?;
        }
        Ok(())
    }

    fn fill_audio(&mut self) {
        let out = &mut self.buf;

        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };

            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
