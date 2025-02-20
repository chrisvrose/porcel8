use crate::{device::instruction::Instruction, util::EmulatorError};
use crate::device::keyboard::Keyboard;
use crate::device::timer::DeviceTimerManager;
use crate::util::{DeviceConfig, EmulatorResult};
use rand::random;
use rand::seq::IteratorRandom;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

pub struct Device {
    pub registers: RegisterFile,
    pub memory: Box<[u8; Self::DEVICE_MEMORY_SIZE]>,
    pub timer: DeviceTimerManager,
    pub stack: Vec<u16>,
    pub frame_buffer: Arc<Mutex<Box<[bool; 64 * 32]>>>,
    pub device_keyboard: Keyboard,
    pub device_config: DeviceConfig
}

impl Device {
    pub const DEVICE_MEMORY_SIZE: usize = 1 << 12;
    pub const FRAME_BUFFER_WIDTH: usize = 64;
    pub const FRAME_BUFFER_HEIGHT: usize = 32;
    pub const FRAME_BUFFER_SIZE: usize = Self::FRAME_BUFFER_WIDTH * Self::FRAME_BUFFER_HEIGHT;
    pub fn new(
        timer: DeviceTimerManager,
        fb: Arc<Mutex<Box<[bool; Device::FRAME_BUFFER_SIZE]>>>,
        device_keyboard: Keyboard,
        device_config: DeviceConfig
    ) -> Device {
        let memory = vec![0u8; Self::DEVICE_MEMORY_SIZE]
            .into_boxed_slice()
            .try_into()
            .unwrap();
        log::trace!("Successfully initiated device memory");
        Device {
            registers: RegisterFile::new(),
            memory,
            frame_buffer: fb,
            stack: Vec::with_capacity(16),
            timer,
            device_keyboard,
            device_config
        }
    }
}

impl Device {
    const FONT_HEIGHT: u16 = 5;
    const FONT_DEFAULT_MEM_LOCATION_START: usize = 0x50;
    const FONT_DEFAULT_MEM_LOCATION_END: usize = 0x9F;
    const ROM_START: usize = 0x200;

    pub fn cycle(&mut self) -> EmulatorResult<()> {
        let time_start = std::time::Instant::now();
        self.device_keyboard.update_keyboard_registers()?;

        let pc = self.registers.pc as usize;
        let instr_slice = self.memory.get(pc..pc + 2).expect(format!("Failed to get memory at {}",pc).as_str());
        self.registers.pc += 2;

        let instruction = Instruction::decode_instruction(instr_slice);
        self.execute_instruction(instruction)?;

        if let Some(throttling_duration) = self.device_config.get_throttling_config() {
            let instruction_time = time_start.elapsed();
            let time_left_to_sleep_for_instruction = throttling_duration.checked_sub(instruction_time).unwrap_or(Duration::ZERO);
            log::trace!("Instruction took {:?}, left with {:?}",instruction_time,time_left_to_sleep_for_instruction);
            sleep(time_left_to_sleep_for_instruction);
        }

        Ok(())
    }
    /// convert the 2 indices into one
    fn get_framebuffer_index(x: usize, y: usize) -> usize {
        y * Self::FRAME_BUFFER_WIDTH + x
    }
    pub fn execute_instruction(&mut self, instruction: Instruction) -> EmulatorResult<()> {
        log::trace!("Executing {:?}, {:?}", &instruction, &self.registers);
        match instruction {
            Instruction::InvalidInstruction => {
                log::info!("Executing passthrough");
                if self.device_config.should_halt_on_invalid() {
                    return Err(EmulatorError::IOError("Caught Invalid Instruction".to_string()));
                }
            },
            Instruction::ClearScreen => {
                let mut frame_buffer = self
                    .frame_buffer
                    .lock()
                    .expect("Failed to grab framebuffer for drawing");
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
                self.registers.v[reg_location] = self.registers.v[reg_location].wrapping_add(value);
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
                let regnum = if self.device_config.is_new_chip8() { x } else { 0 };
                let new_pc = self.registers.v[regnum] as u16 + num;
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
                let (wrapped_addition_result, is_overflow) =
                    left.overflowing_add(self.registers.v[y]);
                self.registers.v[x] = wrapped_addition_result;
                self.set_flag_register(is_overflow);
            }
            Instruction::Sub(x, y) => {
                let left = self.registers.v[x];
                let right = self.registers.v[y];
                let (wrapped_subtraction_result, is_overflow) = left.overflowing_sub(right);
                self.registers.v[x] = wrapped_subtraction_result;
                self.set_flag_register(!is_overflow);
            }
            Instruction::RSub(x, y) => {
                let left = self.registers.v[y];
                let (wrapped_subtraction_result, is_overflow) =
                    left.overflowing_sub(self.registers.v[x]);
                self.registers.v[x] = wrapped_subtraction_result;
                self.set_flag_register(!is_overflow);
            }
            Instruction::RShift(x, y) => {
                if !self.device_config.is_new_chip8() {
                    self.registers.v[x] = self.registers.v[y];
                }
                let val = self.registers.v[x];
                let (shift_res, bit_carry) = Self::shr_1(val);
                self.registers.v[x] = shift_res;
                self.set_flag_register(bit_carry);
            }
            Instruction::LShift(x, y) => {
                if !self.device_config.is_new_chip8() {
                    self.registers.v[x] = self.registers.v[y];
                }
                let left = self.registers.v[x];
                let (res, bit_carry) = Self::shl_1(left);
                self.registers.v[x] = res;
                self.set_flag_register(bit_carry);
            }

            Instruction::FetchDelayTimer(x) => {
                let timer_left = self.timer.poll_value()?;
                self.registers.v[x] = timer_left
            }
            Instruction::SetDelayTimer(x) => {
                let delay_timer_val = self.registers.v[x];
                self.timer.try_set_timer(delay_timer_val)?;
            }
            Instruction::SetSoundTimer(x) => {
                let delay_timer_val = self.registers.v[x];
                self.timer.try_set_sound(delay_timer_val)?;
            }
            Instruction::AddToIndex(x) => {
                let reg_value = self.registers.v[x];
                let index_original = self.registers.i;
                // newer instruction set requires wrapping on 12 bit overflow, and setting vf
                let addn_res = if self.device_config.is_new_chip8() {
                    let overflowing = (reg_value as u16 + index_original) >= 0x1000;
                    self.set_flag_register(overflowing);
                    (reg_value as u16 + index_original) % 0x1000
                } else {
                    reg_value as u16 + index_original
                };
                self.registers.i = addn_res;
            }
            Instruction::GetKey(x) => {
                // if !self.device_keyboard.query_key_down(key_expected) {
                //     self.registers.pc -= 2;
                // }
                let mut possible_presses = (0..=0xfu8).filter(|x|{self.device_keyboard.query_key_down(*x)});
                let pressed = possible_presses.next();
                if let Some(pressed_key) = pressed {

                    self.registers.v[x] = pressed_key;

                } else{
                    self.registers.pc -= 2;
                }
                // let key_expected = self.registers.v[x];
            }
            Instruction::SetIndexToFontCharacter(x) => {
                let requested_char = self.registers.v[x];
                let font_address = Self::FONT_DEFAULT_MEM_LOCATION_START as u16
                    + Self::FONT_HEIGHT * requested_char as u16;
                self.registers.i = font_address;
            }
            Instruction::DoBCDConversion(x) => {
                let mut binary_value_to_decode_temp = self.registers.v[x];
                let unit_digit = binary_value_to_decode_temp % 10;
                binary_value_to_decode_temp /= 10;
                let tens_digit = binary_value_to_decode_temp % 10;
                binary_value_to_decode_temp /= 10;
                let hundreds_digit = binary_value_to_decode_temp % 10;
                binary_value_to_decode_temp /= 10;

                // If this fails, something has gone truly wrong
                assert_eq!(0, binary_value_to_decode_temp);

                let val = [hundreds_digit, tens_digit, unit_digit];
                let index = self.registers.i as usize;
                self.memory[index..(index + 3)].copy_from_slice(&val);
            }
            Instruction::StoreRegistersToMemory(last_reg_to_store) => {
                let reg_slice = &self.registers.v[0..=last_reg_to_store];
                let index = self.registers.i as usize;
                self.memory[index..=(index + last_reg_to_store)].copy_from_slice(reg_slice);
                // Old Chip8 used to use i as a incrementing index
                if !self.device_config.is_new_chip8() {
                    self.registers.i += last_reg_to_store as u16 + 1;
                }
            }
            Instruction::LoadRegistersFromMemory(last_reg_to_load) => {
                let index = self.registers.i as usize;
                let mem_slice = &self.memory[index..=(index + last_reg_to_load)];
                self.registers.v[0..=last_reg_to_load].copy_from_slice(mem_slice);
                // Old Chip8 used to use i as a incrementing index
                if !self.device_config.is_new_chip8() {
                    self.registers.i += last_reg_to_load as u16 + 1;
                }
            }
        };
        Ok(())
    }
    ///
    /// Draw a sprite at location at (x,y) for n pixels long and 8 pixels wide.
    /// Returns whether any pixel was toggled
    fn draw_sprite_at_location(&mut self, x: usize, y: usize, n: u8) -> bool {
        let mut frame_buffer = self
            .frame_buffer
            .lock()
            .expect("Failed to grab framebuffer for drawing");

        let mut is_pixel_toggled_off = false;
        for i in 0..n as usize {
            let index = Self::get_framebuffer_index(x, y + i);
            let slice_from_memory = self.memory[self.registers.i as usize + i];
            // if we are drawing below the screen
            if (y + i) >= Self::FRAME_BUFFER_HEIGHT {
                log::trace!("Overdraw detected, skipping");
                continue;
            }
            for bit_index in (0..8).rev() {
                // if going out of the screen, stop
                if Self::get_framebuffer_index(0, y + i + 1) <= (index + (7 - bit_index)) {
                    break;
                }
                let bit_is_true = (slice_from_memory & (1 << bit_index)) == (1 << bit_index);

                // if the pixel is going to be toggled false, set this flag bit to true
                if frame_buffer[index + (7 - bit_index)] && (bit_is_true) {
                    is_pixel_toggled_off = true;
                }
                frame_buffer[index + (7 - bit_index)] =
                    frame_buffer[index + (7 - bit_index)] ^ (bit_is_true);
            }
        }
        is_pixel_toggled_off
    }
    fn set_flag_register(&mut self, x: bool) {
        self.registers.v[0xf] = if x { 1 } else { 0 }
    }

    pub fn set_default_font(&mut self) {
        const DEFAULT_FONT: [u8; Device::FONT_HEIGHT as usize * 16] = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        log::info!("Loaded default font from memory");
        self.memory[Self::FONT_DEFAULT_MEM_LOCATION_START..=Self::FONT_DEFAULT_MEM_LOCATION_END]
            .copy_from_slice(&DEFAULT_FONT);
    }
    /// load a rom from bytes
    pub fn load_rom(&mut self, rom: &[u8]) {
        log::info!("Loaded ROM from memory");
        self.memory[Self::ROM_START..].copy_from_slice(rom);
    }
    /// Shift right and get carried out bit
    fn shr_1(left: u8) -> (u8, bool) {
        let bit_carry = (left & 0x1) == 0x1;
        return ((left) >> 1, bit_carry);
    }
    /// Shift left, and get carried out bit
    fn shl_1(left: u8) -> (u8, bool) {
        let bit_carry = (left & 0x80) == 0x80;
        let left = left & 0x7f;
        return ((left & 0x7f) << 1, bit_carry);
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
