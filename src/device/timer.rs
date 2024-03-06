
use std::sync::{Arc, Mutex};
use std::sync::mpsc::SendError;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;

use crate::util::EmulatorResult;

/// Manages the timer and the sound timer
pub struct TimerManager {
    timer_left: Arc<Mutex<u8>>,
    sound_left: Arc<Mutex<u8>>,
    join_handle: Option<(JoinHandle<()>, std::sync::mpsc::Sender<()>)>,
}

impl TimerManager {
    pub const TIMER_THREAD_NAME: &'static str = "Timer";
    pub fn new() -> TimerManager {
        TimerManager {
            timer_left: Arc::new(Mutex::default()),
            sound_left: Arc::new(Mutex::default()),
            join_handle: None,
        }
    }
    pub fn start(&mut self) -> Arc<Mutex<u8>> {
        let timer_left_ref = self.timer_left.clone();
        let sound_timer_ref = self.sound_left.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        // A 1/60th second

        let res = std::thread::Builder::new().name(Self::TIMER_THREAD_NAME.into()).spawn(move || {
            loop {
                let val = receiver.try_recv();
                if let Ok(()) = val {
                    break;
                } else if let Err(std::sync::mpsc::TryRecvError::Disconnected) = val {
                    panic!("Disconnected");
                }
                {
                    let mut timer_lock = timer_left_ref.lock().expect("Failed to lock");
                    let mut sound_lock = sound_timer_ref.lock().expect("Failed to lock");
                    if *timer_lock > 0 {
                        *timer_lock -= 1;
                    }
                    if *sound_lock > 0 {
                        log::trace!("Beep!");
                        *sound_lock -= 1;
                    }
                }
                sleep(Duration::from_secs_f32(1f32 / 60f32));
            }
        }).expect("Failed to start timer thread");
        self.join_handle = Some((res, sender));
        self.sound_left.clone()
    }

    /// Set a timer down tick from `val`
    pub fn try_set_timer(&self, val: u8) -> EmulatorResult<()> {
        let mut timer_val = self.timer_left.lock()?;
        *timer_val = val;
        Ok(())
    }

    pub fn try_set_sound(&self, val: u8) -> EmulatorResult<()> {
        let mut timer_val = self.sound_left.lock()?;
        *timer_val = val;
        Ok(())
    }

    pub fn poll_value(&self) -> EmulatorResult<u8> {
        let res = self.timer_left.lock()?;
        Ok(res.clone())
    }

    pub fn stop(self) {
        if let Some((u, _)) = self.join_handle {
            u.join().expect("Failed to close thread");
        } else {
            log::warn!("Nothing present!");
        }
    }
    pub fn send_stop_signal(&mut self) {
        if let Some((_, x)) = &self.join_handle {
            match x.send(()) {
                Ok(_) => {
                    log::trace!("Sent stop Signal")
                }
                Err(SendError(_)) => {
                    log::info!("Thread already stopped!");
                }
            };
        } else {
            log::warn!("Nothing present!");
        }
    }
}
