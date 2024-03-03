use byteorder::{BigEndian, ByteOrder};
use crate::device::instruction::Instruction::{AddValueToRegister, ClearScreen, Draw, JumpTo, PassThrough, SetIndex, SetRegister};

#[derive(Eq, PartialEq, Debug)]
pub enum Instruction {
    /// Blanket instruction that does nothing
    PassThrough,
    /// 00E0 - Clear the screen
    ClearScreen,
    /// 1NNN - Jump to location
    JumpTo(u16),
    /// 6XNN - Set register to value
    SetRegister(usize, u8),
    /// 7XNN - Add value to register
    AddValueToRegister(usize, u8),
    /// ANNN - Set index value
    SetIndex(u16),
    ///
    Draw(usize, usize, u8),
}

impl Instruction {
    pub fn decode_instruction(location: &[u8]) -> Instruction {
        assert_eq!(location.len(), 2);
        let instruction = BigEndian::read_u16(location);
        let outer_instruction_nibble = (instruction & 0xF000) >> 12;
        match outer_instruction_nibble {
            0x0 if instruction == 0xe0 => {
                ClearScreen
            }
            0x0 => {
                log::warn!("Ignoring unsupported instruction {}",instruction);
                PassThrough
            }
            0x1 => {
                JumpTo(instruction & 0xfff)
            }
            0x6 => {
                SetRegister(((instruction & 0x0f00) >> 8) as usize, (instruction & 0xff) as u8)
            }
            0x7 => {
                AddValueToRegister(((instruction & 0x0f00) >> 8) as usize, (instruction & 0xff) as u8)
            }
            0xA => {
                SetIndex(instruction & 0xfff)
            }
            0xD => {
                let x = (instruction & 0xf00) >> 8;
                let y = (instruction & 0xf0) >> 4;
                let n = instruction & 0xf;
                Draw(x as usize, y as usize, n as u8)
            }
            0x8 => {
                todo!("Arithmetic instructions pending")
            }
            _ => {
                todo!("Unimplemented instruction")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::device::instruction::Instruction;
    use crate::device::instruction::Instruction::{AddValueToRegister, ClearScreen, Draw, JumpTo, SetIndex, SetRegister};

    #[test]
    fn test_clear_screen() {
        let instruction_bytes = 0x00e0_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, ClearScreen);
    }

    #[test]
    #[should_panic]
    fn test_other_0x0nnn_instructions_panic() {
        let instruction_bytes = 0x00f0_u16.to_be_bytes();
        Instruction::decode_instruction(&instruction_bytes);
    }


    #[test]
    fn test_jump_to_instruction_1() {
        let instruction_bytes = 0x1123_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, JumpTo(0x123));
    }

    #[test]
    fn test_jump_to_instruction_2() {
        let instruction_bytes = 0x1faf_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, JumpTo(0xfaf));
    }

    #[test]
    fn test_set_register_instruction() {
        let instruction_bytes = 0x6a00_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, SetRegister(10, 0));
    }

    #[test]
    fn test_set_register_instruction_2() {
        let instruction_bytes = 0x6f23_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, SetRegister(15, 0x23));
    }

    #[test]
    fn test_add_register_instruction_2() {
        let instruction_bytes = 0x7f23_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, AddValueToRegister(15, 0x23));
    }

    #[test]
    fn test_set_index() {
        let instruction_bytes = 0xafaf_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, SetIndex(0xfaf));
    }

    #[test]
    fn test_draw() {
        let instruction_bytes = 0xdfab_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, Draw(0xf, 0xa, 0xb))
    }
}