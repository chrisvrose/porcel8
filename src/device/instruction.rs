use byteorder::{BigEndian, ByteOrder};

#[derive(Eq, PartialEq, Debug)]
pub enum Instruction {
    /// Blanket instruction that does nothing
    PassThrough,
    /// 00E0 - Clear the screen
    ClearScreen,
    /// 00EE - Return from procedure
    ReturnFromProcedure,
    /// 1NNN - Jump to location
    JumpTo(u16),
    /// 2NNN - Link and jump
    JumpAndLink(u16),
    /// 3XNN - If register equals number, Skip next
    ConditionalEqSkipNext(usize, u8),
    /// 4XNN - If register equals number, Skip next
    ConditionalInEqSkipNext(usize, u8),
    /// 5XY0 - If registers equal, Skip next
    ConditionalEqRegisterSkipNext(usize, usize),
    /// 6XNN - Set register to value
    SetRegister(usize, u8),
    /// 7XNN - Add value to register
    AddValueToRegister(usize, u8),
    /// 9XY0 - If registers not equal, skip next
    ConditionalInEqRegisterSkipNext(usize, usize),
    /// ANNN - Set index value
    SetIndex(u16),
    /// DXYN - Draw pixels at xy pointed by register for n bytes long
    Draw(usize, usize, u8),
    
    
    // ALU operations going ahead
    /// 8XY0 - x=y
    Set(usize,usize),
    /// 8XY1 - x|=y
    Or(usize,usize),
    /// 8XY2 - x|=y
    And(usize,usize),
    /// 8XY3 - x^=y
    Xor(usize,usize),
    /// 8XY4 - x+=y
    Add(usize,usize),
    /// 8XY5 - x-=y
    Sub(usize,usize),
    /// 8XY6 - (x=y)?, x>>=1 
    RShift(usize,usize),
    /// 8XY7 - x=y-x
    RSub(usize,usize),
    /// 8XYE - (x=y)?, x<<=1
    LShift(usize,usize),
}

impl Instruction {
    pub fn decode_instruction(location: &[u8]) -> Instruction {
        assert_eq!(location.len(), 2);
        let instruction = BigEndian::read_u16(location);
        let outer_instruction_nibble = (instruction & 0xF000) >> 12;
        match outer_instruction_nibble {
            0x0 if instruction == 0xe0 => {
                Instruction::ClearScreen
            }
            0x0 if instruction == 0xee => {
                Instruction::ReturnFromProcedure
            }
            0x0 => {
                log::warn!("Ignoring unsupported instruction {}",instruction);
                Instruction::PassThrough
            }
            0x1 => {
                Instruction::JumpTo(instruction & 0xfff)
            }
            0x2 => {
                Instruction::JumpAndLink(instruction & 0xfff)
            }
            0x3 => {
                let register = (instruction & 0xf00) >> 8;
                let val = instruction & 0xff;
                Instruction::ConditionalEqSkipNext(register as usize, val as u8)
            }
            0x4 => {
                let register = (instruction & 0xf00) >> 8;
                let val = instruction & 0xff;
                Instruction::ConditionalInEqSkipNext(register as usize, val as u8)
            }
            0x5 => {
                let registerx = (instruction & 0xf00) >> 8;
                let registery = (instruction & 0xf0) >> 4;

                Instruction::ConditionalEqRegisterSkipNext(registerx as usize, registery as usize)
            }
            0x6 => {
                Instruction::SetRegister(((instruction & 0x0f00) >> 8) as usize, (instruction & 0xff) as u8)
            }
            0x7 => {
                Instruction::AddValueToRegister(((instruction & 0x0f00) >> 8) as usize, (instruction & 0xff) as u8)
            }
            0x9 =>{
                let registerx = (instruction & 0xf00) >> 8;
                let registery = (instruction & 0xf0) >> 4;

                Instruction::ConditionalInEqRegisterSkipNext(registerx as usize, registery as usize)
            }
            0xA => {
                Instruction::SetIndex(instruction & 0xfff)
            }
            0xD => {
                let x = (instruction & 0xf00) >> 8;
                let y = (instruction & 0xf0) >> 4;
                let n = instruction & 0xf;
                Instruction::Draw(x as usize, y as usize, n as u8)
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
    use crate::device::instruction::Instruction::*;

    #[test]
    fn test_clear_screen() {
        let instruction_bytes = 0x00e0_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, ClearScreen);
    }

    #[test]
    fn test_procedure_return() {
        let instruction_bytes = 0x00ee_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, ReturnFromProcedure);
    }

    #[test]
    fn test_other_0x0nnn_instructions_passthrough() {
        for instruction_hex in [0xf0u16, 0x0, 0x1, 0x10] {
            let instruction_bytes = instruction_hex.to_be_bytes();
            let instruction = Instruction::decode_instruction(&instruction_bytes);
            assert_eq!(instruction, PassThrough)
        }
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
    fn test_link_jump() {
        let instruction_bytes = 0x2afa_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, JumpAndLink(0xafa));
    }

    #[test]
    fn test_conditional_equal_skip() {
        let instruction_bytes = 0x3fad_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, ConditionalEqSkipNext(0xf, 0xad));
    }

    #[test]
    fn test_conditional_in_equal_skip() {
        let instruction_bytes = 0x4fea_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, ConditionalInEqSkipNext(0xf, 0xea));
    }

    #[test]
    fn test_conditional_register_equal_skip() {
        let instruction_bytes = 0x5fa0_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, ConditionalEqRegisterSkipNext(0xf, 0xa));
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
    fn test_conditional_register_in_equal_skip() {
        let instruction_bytes = 0x9af0_u16.to_be_bytes();
        let ins = Instruction::decode_instruction(&instruction_bytes);
        assert_eq!(ins, ConditionalInEqRegisterSkipNext(0xa, 0xf));
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