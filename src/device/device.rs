use std::ops::SubAssign;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use rand::random;
use crate::device::instruction::Instruction;
use crate::device::keyboard::Keyboard;
use crate::device::timer::Timer;
use crate::util::EmulatorResult;


pub struct Device {
    pub registers: RegisterFile,
    pub memory: Box<[u8; Self::DEVICE_MEMORY_SIZE]>,
    pub timer: Timer,
    pub stack: Vec<u16>,
    pub frame_buffer: Arc<Mutex<Box<[bool; 64 * 32]>>>,
    pub super_chip8_mode: bool,
    pub device_keyboard: Keyboard,
}

impl Device {
    pub const DEVICE_MEMORY_SIZE: usize = 1 << 12;
    pub const FRAME_BUFFER_WIDTH: usize = 64;
    pub const FRAME_BUFFER_HEIGHT: usize = 32;
    pub const FRAME_BUFFER_SIZE: usize = Self::FRAME_BUFFER_WIDTH * Self::FRAME_BUFFER_HEIGHT;
    pub fn new(timer: Timer, fb: Arc<Mutex<Box<[bool; Device::FRAME_BUFFER_SIZE]>>>, device_keyboard: Keyboard) -> Device {
        let memory = vec![0u8; Self::DEVICE_MEMORY_SIZE].into_boxed_slice().try_into().unwrap();
        log::trace!("Successfully initiated device memory");
        Device {
            registers: RegisterFile::new(),
            memory,
            frame_buffer: fb,
            stack: Vec::with_capacity(16),
            timer,
            super_chip8_mode: false,
            device_keyboard,
        }
    }
}

impl Device {
    const DEFAULT_FONT: [u8; 5 * 16] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];
    const FONT_DEFAULT_MEM_LOCATION_START: usize = 0x50;
    const FONT_DEFAULT_MEM_LOCATION_END: usize = 0x9F;
    const ROM_START: usize = 0x200;
    pub fn cycle(&mut self) -> EmulatorResult<()> {
        self.device_keyboard.update_keyboard()?;

        let pc = self.registers.pc as usize;
        let instr_slice = self.memory.get(pc..pc + 2).expect("Failed to get memory");
        self.registers.pc += 2;

        let instruction = Instruction::decode_instruction(instr_slice);
        self.execute_instruction(instruction);
        Ok(())
    }
    /// convert the 2 indices into one
    fn get_framebuffer_index(x: usize, y: usize) -> usize {
        y * Self::FRAME_BUFFER_WIDTH + x
    }
    pub fn execute_instruction(&mut self, instruction: Instruction) {
        // thread::sleep(Duration::from_millis(250));
        log::trace!("Executing {:?}, {:?}",&instruction,&self.registers);
        match instruction {
            Instruction::PassThrough => {
                log::info!("Executing passthrough");
            }
            Instruction::ClearScreen => {
                let mut frame_buffer = self.frame_buffer.lock().expect("Failed to grab framebuffer for drawing");
                for pixel in frame_buffer.iter_mut() {
                    *pixel = false;
                }
                log::trace!("ClearScreen")
            }
            Instruction::JumpTo(new_pc) => {
                // hint that we're jumping back to self
                self.registers.pc = new_pc;
            }
            Instruction::SetRegister(reg_location, value) => {
                self.registers.v[reg_location] = value;
            }
            Instruction::AddValueToRegister(reg_location, value) => {
                self.registers.v[reg_location] += value;
            }
            Instruction::SetIndex(value) => {
                self.registers.i = value;
            }
            Instruction::Draw(regx, regy, n) => {
                let x = self.registers.v[regx] as usize;
                let y = self.registers.v[regy] as usize;
                let toggle_state = self.draw_sprite_at_location(x, y, n);
                self.set_flag_register(toggle_state);
            }
            Instruction::JumpAndLink(jump_location) => {
                self.stack.push(self.registers.pc);
                self.registers.pc = jump_location;
            }
            Instruction::ReturnFromProcedure => {
                let old_pc = self.stack.pop().expect("Expected value on stack pop");
                self.registers.pc = old_pc;
            }

            Instruction::ConditionalEqSkipNext(regx, num) => {
                if self.registers.v[regx] == num {
                    self.registers.pc += 2;
                }
            }
            Instruction::ConditionalInEqSkipNext(regx, num) => {
                if self.registers.v[regx] != num {
                    self.registers.pc += 2;
                }
            }
            Instruction::ConditionalEqRegisterSkipNext(regx, regy) => {
                if self.registers.v[regx] == self.registers.v[regy] {
                    self.registers.pc += 2;
                }
            }
            Instruction::ConditionalInEqRegisterSkipNext(regx, regy) => {
                if self.registers.v[regx] != self.registers.v[regy] {
                    self.registers.pc += 2;
                }
            }
            Instruction::JumpWithOffset(x, num) => {
                let regnum = if self.super_chip8_mode { x } else { 0 };
                let new_pc = self.registers.v[regnum] as u16 + num ;
                self.registers.pc = new_pc;
            }
            Instruction::RandomAnd(dest, n) => {
                self.registers.v[dest] = random::<u8>() & n;
            }
            Instruction::SkipIfKeyPressed(x) => {
                let key_press_expected_for = self.registers.v[x];
                if self.device_keyboard.query_key_down(key_press_expected_for) {
                    self.registers.pc += 2;
                }
            }
            Instruction::SkipIfKeyNotPressed(x) => {
                let key_press_expected_for = self.registers.v[x];
                if !self.device_keyboard.query_key_down(key_press_expected_for) {
                    self.registers.pc += 2;
                }
            }
            Instruction::Set(x, y) => {
                self.registers.v[x] = self.registers.v[y];
            }
            Instruction::Or(x, y) => {
                self.registers.v[x] |= self.registers.v[y];
            }
            Instruction::And(x, y) => {
                self.registers.v[x] &= self.registers.v[y];
            }
            Instruction::Xor(x, y) => {
                self.registers.v[x] ^= self.registers.v[y];
            }
            Instruction::Add(x, y) => {
                let left = self.registers.v[x];
                let (wrapped_addition_result, is_overflow) = left.overflowing_add(self.registers.v[y]);
                self.registers.v[x] = wrapped_addition_result;
                self.set_flag_register(is_overflow);
            }
            Instruction::Sub(x, y) => {
                let left = self.registers.v[x];
                let (wrapped_subtraction_result, is_overflow) = left.overflowing_sub(self.registers.v[y]);
                self.registers.v[x] = wrapped_subtraction_result;
                self.set_flag_register(is_overflow);
            }
            Instruction::RSub(x, y) => {
                let left = self.registers.v[y];
                let (wrapped_subtraction_result, is_overflow) = left.overflowing_sub(self.registers.v[x]);
                self.registers.v[x] = wrapped_subtraction_result;
                self.set_flag_register(is_overflow);
            }
            Instruction::RShift(x, y) => {
                if(self.super_chip8_mode){
                    self.registers.v[x] = self.registers.v[y];
                }
                let (shift_res,did_overflow) = self.registers.v[x].overflowing_shr(1);
                self.registers.v[x] = shift_res;
                self.set_flag_register(did_overflow);
            }
            Instruction::LShift(x, y) => {
                if(self.super_chip8_mode){
                    self.registers.v[x] = self.registers.v[y];
                }
                let (shift_res,did_overflow) = self.registers.v[x].overflowing_shl(1);
                self.registers.v[x] = shift_res;
                self.set_flag_register(did_overflow);
            },
            _=>{
                todo!()
            }
        };
    }
    ///
    /// Draw a sprite at location at (x,y) for n pixels long and 8 pixels wide.
    /// Returns whether any pixel was toggled
    fn draw_sprite_at_location(&mut self, x: usize, y: usize, n: u8) -> bool {
        let mut frame_buffer = self.frame_buffer.lock().expect("Failed to grab framebuffer for drawing");

        let mut is_pixel_toggled_off = false;
        for i in 0..n as usize {
            let index = Self::get_framebuffer_index(x, y + i);
            let slice_from_memory = self.memory[self.registers.i as usize + i];

            for bit_index in (0..8).rev() {
                // if i'm going to the next line, stop
                if Self::get_framebuffer_index(0, y + 1) == index {
                    break;
                }
                let bit_is_true = (slice_from_memory & (1 << bit_index)) == (1 << bit_index);

                // if the pixel is going to be toggled false, set this flag bit to true
                if frame_buffer[index + (7 - bit_index)] && (bit_is_true) { is_pixel_toggled_off = true; }
                frame_buffer[index + (7 - bit_index)] = frame_buffer[index + (7 - bit_index)] ^ (bit_is_true);
            }
        }
        is_pixel_toggled_off
    }
    fn set_flag_register(&mut self, x: bool) {
        self.registers.v[0xf] = if x { 1 } else { 0 }
    }

    pub fn set_default_font(&mut self) {
        log::info!("Loaded default font from memory");
        self.memory[Self::FONT_DEFAULT_MEM_LOCATION_START..=Self::FONT_DEFAULT_MEM_LOCATION_END].copy_from_slice(&Self::DEFAULT_FONT);
    }
    /// load a rom from bytes
    pub fn load_rom(&mut self, rom: &[u8]) {
        log::info!("Loaded ROM from memory");
        self.memory[Self::ROM_START..].copy_from_slice(rom);
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.timer.send_stop_signal()
    }
}

#[derive(Debug)]
pub struct RegisterFile {
    pub v: [u8; 0x10],
    /// program counter - only u12 technically.
    pub pc: u16,
    /// stack pointer
    pub i: u16,
}

impl RegisterFile {
    pub const DEFAULT_PC_VALUE: u16 = Device::ROM_START as u16;
    pub fn new() -> RegisterFile {
        RegisterFile {
            v: [0; 0x10],
            pc: Self::DEFAULT_PC_VALUE,
            i: 0,
        }
    }
}
