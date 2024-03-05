use std::sync::mpsc::TryRecvError;
use crate::util::{EmulatorError, EmulatorResult};


// display thread sends these to the device thread.

// device thread updates on key up and down?


pub struct Keyboard {
    bitflags: u16,
    keyboard_event_receiver: std::sync::mpsc::Receiver<KeyboardEvent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyboardEvent {
    KeyUp(u8),
    KeyDown(u8),
}

impl Keyboard {
    pub fn new(keyboard_event_receiver: std::sync::mpsc::Receiver<KeyboardEvent>) -> Keyboard {
        Keyboard {
            bitflags: 0,
            keyboard_event_receiver,
        }
    }

    /// Update keyboard based on pending keyboard events
    pub fn update_keyboard(&mut self) -> EmulatorResult<()> {
        loop {
            let keyboard_event_recv_res = self.keyboard_event_receiver.try_recv();
            match keyboard_event_recv_res {
                Ok(event) => {
                    log::warn!("Processing {:?}",event);
                    self.update_keyboard_state(event);
                }
                Err(TryRecvError::Empty) => {
                    break Ok(());
                }
                Err(TryRecvError::Disconnected) => {
                    break Err(EmulatorError::IOError("Keyboard updater disconnected".into()));
                }
            }
        }
    }


    /// Query if key is down
    pub fn query_key_down(&self, key_num: u8) -> bool {
        (self.bitflags | 1 << key_num) == (1 << key_num)
    }

    fn update_keyboard_state(&mut self, keyboard_event: KeyboardEvent) {
        match keyboard_event {
            KeyboardEvent::KeyUp(key) => { self.key_up(key) }
            KeyboardEvent::KeyDown(key) => { self.key_down(key) }
        }
    }
    fn key_down(&mut self, x: u8) {
        self.bitflags |= 1 << x;
        log::trace!("Key Down - state {}",self.bitflags);
    }
    fn key_up(&mut self, x: u8) {
        self.bitflags &= !((1 << x) as u16);
        log::debug!("Key Up - state {}",self.bitflags);
    }
}