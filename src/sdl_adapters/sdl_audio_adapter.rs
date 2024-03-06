use std::sync::{Arc, Mutex};
use sdl2::audio::AudioQueue;
use crate::device::timer::DeviceTimerManager;
use crate::util::EmulatorResult;

/// An Audio adapter using `AudioQueue`. Generates a square wave of specified frequency
pub struct SdlAudioAdapter {
    sound_timer: Arc<Mutex<u8>>,
    phase_inc: f32,
    phase: f32,
    volume: f32,
    audio_queue: AudioQueue<f32>,
    internal_buffer: Vec<f32>,
}

impl SdlAudioAdapter {
    /// Number of samples per second
    pub const SAMPLING_FREQ:i32 = 15360;
    pub const SAMPLES_PER_FRAME: usize = (Self::SAMPLING_FREQ as usize / 60) * 2;
    pub fn new_timers(freq: f32,
                 volume: f32,
                 audio_queue: AudioQueue<f32>) ->(DeviceTimerManager,SdlAudioAdapter){
        let device_sound_timer = Arc::new(Mutex::default());
        let device_timer_manager = DeviceTimerManager::new(device_sound_timer.clone());
        let sdl_audio_adapter = SdlAudioAdapter::new(device_sound_timer,freq,volume,audio_queue);
        (device_timer_manager, sdl_audio_adapter)
    }
    fn new(sound_timer: Arc<Mutex<u8>>,
               freq: f32,
               volume: f32,
               audio_queue: AudioQueue<f32>) -> SdlAudioAdapter {
        audio_queue.resume();
        // ensure frequency isn't too low
        assert!(((2.0*freq) as i32) < Self::SAMPLING_FREQ);
        SdlAudioAdapter {
            sound_timer,
            internal_buffer: vec![0f32; Self::SAMPLES_PER_FRAME],
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
            self.audio_queue.queue_audio(&self.internal_buffer)?;
        }
        Ok(())
    }

    fn fill_audio(&mut self) {
        let out = &mut self.internal_buffer;

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
