use byteorder::{BigEndian, ByteOrder};
use crate::device::cpu::Instruction::{ClearScreen, JumpTo};

#[derive(Eq, PartialEq, Debug)]
enum Instruction{
    ClearScreen,
    /// Jump to location
    JumpTo(u16),
    SetRegister(usize,u8),
    AddValueToRegister(usize,u8),
    SetIndex(u16),
    ///
    DRAW(usize,usize,u8)
}
impl Instruction{
    pub fn parse_fetched_instruction(location: &[u8])->Instruction{
        assert_eq!(location.len(),2);
        let instruction = BigEndian::read_u16(location);

        if instruction == 0xe0{
            return ClearScreen;
        } else if (instruction & 0x1000)==0x1000 {
            return JumpTo(instruction & 0xfff);
        }
        todo!();
    }
}

#[cfg(test)]
mod tests{
    use crate::device::cpu::Instruction;
    use crate::device::cpu::Instruction::{ClearScreen, JumpTo};

    #[test]
    fn test_clear_screen(){
        let instruction_bytes = 0x00e0_u16.to_be_bytes();
        let ins = Instruction::parse_fetched_instruction(&instruction_bytes);
        assert_eq!(ins,ClearScreen);
    }
    #[test]
    fn test_jump_to_instruction_1(){
        let instruction_bytes = 0x1123_u16.to_be_bytes();
        let ins = Instruction::parse_fetched_instruction(&instruction_bytes);
        assert_eq!(ins,JumpTo(0x123));
    }
    #[test]
    fn test_jump_to_instruction_2(){
        let instruction_bytes = 0x1faf_u16.to_be_bytes();
        let ins = Instruction::parse_fetched_instruction(&instruction_bytes);
        assert_eq!(ins,JumpTo(0xfaf));
    }
}