use sdl2::keyboard::Keycode;

/// get index of key pressed. 0..9+A..F provides a u8
pub fn get_key_index(p0: Keycode) -> Option<u8> {
    match p0 {
        Keycode::Kp0 => Some(0x0),
        Keycode::Kp1 => Some(0x1),
        Keycode::Kp2 => Some(0x2),
        Keycode::Kp3 => Some(0x3),
        Keycode::Kp4 => Some(0x4),
        Keycode::Kp5 => Some(0x5),
        Keycode::Kp6 => Some(0x6),
        Keycode::Kp7 => Some(0x7),
        Keycode::Kp8 => Some(0x8),
        Keycode::Kp9 => Some(0x9),
        Keycode::A => Some(0xA),
        Keycode::B => Some(0xB),
        Keycode::C => Some(0xC),
        Keycode::D => Some(0xD),
        Keycode::E => Some(0xE),
        Keycode::F => Some(0xF),
        _ => None
    }
}
