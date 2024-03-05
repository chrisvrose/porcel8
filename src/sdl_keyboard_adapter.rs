use std::sync::mpsc::Sender;
use crate::device::keyboard::{Keyboard, KeyboardEvent};
use crate::device::keyboard::KeyboardEvent::{KeyDown, KeyUp};
use crate::util::EmulatorResult;

#[derive(Debug)]
pub struct SdlKeyboardAdapter {
    keyboard_event_sender: Sender<KeyboardEvent>,
}

impl SdlKeyboardAdapter {
    fn new(keyboard_event_sender: Sender<KeyboardEvent>) -> SdlKeyboardAdapter {
        SdlKeyboardAdapter {
            keyboard_event_sender
        }
    }
    /// Creates a paired keyboard and adapter.
    pub fn new_keyboard()->(SdlKeyboardAdapter, Keyboard){
        let (sender,receiver) = std::sync::mpsc::channel();
        let sdl2_kb_adapter = Self::new(sender);
        let device_kb = Keyboard::new(receiver);
        (sdl2_kb_adapter,device_kb)
    }

    pub fn send_key_up(&self, keycode: u8) -> EmulatorResult<u8> {
        self.keyboard_event_sender.send(KeyUp(keycode))?;
        Ok(keycode)
    }
    pub fn send_key_down(&self, keycode: u8) -> EmulatorResult<u8> {
        self.keyboard_event_sender.send(KeyDown(keycode))?;
        Ok(keycode)
    }

}
