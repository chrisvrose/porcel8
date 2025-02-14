use std::sync::mpsc::Sender;
use crate::device::keyboard::{Key, Keyboard, KeyboardEvent};
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

    pub fn process_key_up(&self, keycode: sdl2::keyboard::Keycode) -> EmulatorResult<()> {
        log::debug!("Sending Key up {}",keycode);
        if let Some(key) = Self::keycode_to_key(keycode){
            self.keyboard_event_sender.send(KeyUp(key))?;
        }
        Ok(())
    }
    pub fn process_key_down(&self, keycode: sdl2::keyboard::Keycode) -> EmulatorResult<()> {
        log::trace!("Sending Key down {}",keycode);
        if let Some(key) = Self::keycode_to_key(keycode){
            self.keyboard_event_sender.send(KeyDown(key))?;
        }
        Ok(())
    }
    /// Key map configuration
    pub fn keycode_to_key(keycode: sdl2::keyboard::Keycode) -> Option<Key>{
        match keycode {
            sdl2::keyboard::Keycode::X=>Some(Key::K0),
            sdl2::keyboard::Keycode::Num1=>Some(Key::K1),
            sdl2::keyboard::Keycode::Num2=>Some(Key::K2),
            sdl2::keyboard::Keycode::Num3=>Some(Key::K3),
            sdl2::keyboard::Keycode::Q=>Some(Key::K4),
            sdl2::keyboard::Keycode::W=>Some(Key::K5),
            sdl2::keyboard::Keycode::E=>Some(Key::K6),
            sdl2::keyboard::Keycode::A=>Some(Key::K7),
            sdl2::keyboard::Keycode::S=>Some(Key::K8),
            sdl2::keyboard::Keycode::D=>Some(Key::K9),
            sdl2::keyboard::Keycode::Z=>Some(Key::KA),
            sdl2::keyboard::Keycode::C=>Some(Key::KB),
            sdl2::keyboard::Keycode::Num4=>Some(Key::KC),
            sdl2::keyboard::Keycode::R=>Some(Key::KD),
            sdl2::keyboard::Keycode::F=>Some(Key::KE),
            sdl2::keyboard::Keycode::V=>Some(Key::KF),
            _=>None
        }
    }

}

#[cfg(test)]
mod tests{
    use crate::device::keyboard::Key;

    use super::SdlKeyboardAdapter;

    #[test]
    fn test_keycode_to_key(){
        assert_eq!(Some(Key::K7), SdlKeyboardAdapter::keycode_to_key(sdl2::keyboard::Keycode::A));
    }
    #[test]
    fn test_keycode_to_key_no_key(){
        assert_eq!(None, SdlKeyboardAdapter::keycode_to_key(sdl2::keyboard::Keycode::L));
    }
}
