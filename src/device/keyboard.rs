use std::sync::mpsc::TryRecvError;
use crate::util::{EmulatorError, EmulatorResult};


/// An emulated keyboard that receives keyboard data as input
pub struct Keyboard {
    /// Current keyboard state
    bitflags: u16,
    /// Receives keyboard events from main thread
    keyboard_event_receiver: std::sync::mpsc::Receiver<KeyboardEvent>,
}
#[derive(Eq,PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum Key{
    K0=0,K1,K2,K3,K4,K5,K6,K7,K8,K9,KA,KB,KC,KD,KE,KF
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyboardEvent {
    KeyUp(Key),
    KeyDown(Key),
}

impl Keyboard {
    pub fn new(keyboard_event_receiver: std::sync::mpsc::Receiver<KeyboardEvent>) -> Keyboard {
        Keyboard {
            bitflags: 0,
            keyboard_event_receiver,
        }
    }

    /// Update keyboard based on pending keyboard events.
    /// If no events are presents, it will return without any action.
    pub fn update_keyboard_registers(&mut self) -> EmulatorResult<()> {
        loop {
            let keyboard_event_recv_res = self.keyboard_event_receiver.try_recv();
            match keyboard_event_recv_res {
                Ok(event) => {
                    log::debug!("Processing {:?}",event);
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
        (self.bitflags & (1 << key_num)) == (1 << key_num)
    }

    fn update_keyboard_state(&mut self, keyboard_event: KeyboardEvent) {
        match keyboard_event {
            KeyboardEvent::KeyUp(key) => {
                self.bitflags &= !((1u16 << (key as u16)) as u16);
            }
            KeyboardEvent::KeyDown(key) => {
                self.bitflags |= 1 << (key as u16);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Keyboard};

    #[test]
    fn test_key_assignment(){
        assert_eq!(0,Key::K0 as u16);
        assert_eq!(15,Key::KF as u16);
    }

    #[test]
    fn test_key_at_nothing_pressed(){
        let (_sender,receiver) = std::sync::mpsc::sync_channel(1);
        let keyboard = Keyboard::new(receiver);
        assert_no_key_pressed(&keyboard);

    }

    #[test]
    fn test_key_down_then_up(){
        let (sender,receiver) = std::sync::mpsc::sync_channel(1);
        let mut keyboard = Keyboard::new(receiver);
        assert_no_key_pressed(&keyboard);


        sender.try_send(super::KeyboardEvent::KeyDown(Key::K0)).expect("Could not send");
        keyboard.update_keyboard_registers().expect("Could not update keyboard");


        assert_eq!(1,keyboard.bitflags);
        assert_eq!(true,keyboard.query_key_down(0));
        for i in 1..=0xF {
            assert_eq!(false,keyboard.query_key_down(i));
        }



        sender.try_send(super::KeyboardEvent::KeyUp(Key::K0)).expect("Could not send");
        keyboard.update_keyboard_registers().expect("Could not update keyboard");
        assert_no_key_pressed(&keyboard);
    }

    fn assert_no_key_pressed(keyboard: &Keyboard){
        assert_eq!(0,keyboard.bitflags);
        for i in 0..=0xF {
            assert_eq!(false,keyboard.query_key_down(i),"Failed to match at index {}, bitflags at {}",i,keyboard.bitflags);
        }
    }



}
