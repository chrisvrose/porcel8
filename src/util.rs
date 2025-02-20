use crate::device::keyboard::KeyboardEvent;
use sdl2::video::WindowBuildError;
use sdl2::IntegerOrSdlError;
use std::sync::mpsc::SendError;
use std::sync::PoisonError;
use std::time::Duration;

pub type EmulatorResult<T> = Result<T, EmulatorError>;

/// CHIP-8 Device configuration
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DeviceConfig {
    is_new_chip8: bool,
    halt_on_invalid: bool,
    /// None if disabled, target instruction time otherwise
    throttling_time: Option<Duration>,
}

impl DeviceConfig {
    pub fn new(
        is_new_chip8: bool,
        halt_on_invalid: bool,
        do_instruction_throttling: bool,
        ips_throttling_rate: u64,
    ) -> DeviceConfig {
        DeviceConfig {
            is_new_chip8,
            halt_on_invalid,
            throttling_time: if do_instruction_throttling {
                Some(Duration::from_micros(1_000_000 / ips_throttling_rate))
            } else {
                None
            },
        }
    }
    pub fn is_new_chip8(&self) -> bool {
        self.is_new_chip8
    }
    pub fn should_halt_on_invalid(&self) -> bool {
        self.halt_on_invalid
    }
    pub fn get_throttling_config(&self) -> Option<Duration> {
        self.throttling_time
    }
}

#[derive(Clone, Debug)]
pub enum EmulatorError {
    SdlError(String),
    IOError(String),
    MutexInvalidState(String),
}

impl From<String> for EmulatorError {
    fn from(value: String) -> Self {
        Self::SdlError(value)
    }
}

impl From<WindowBuildError> for EmulatorError {
    fn from(value: WindowBuildError) -> Self {
        Self::SdlError(value.to_string())
    }
}

impl From<IntegerOrSdlError> for EmulatorError {
    fn from(value: IntegerOrSdlError) -> Self {
        match value {
            IntegerOrSdlError::IntegerOverflows(x, y) => Self::SdlError(format!("{} - {}", x, y)),
            IntegerOrSdlError::SdlError(str) => Self::SdlError(str),
        }
    }
}
impl From<std::io::Error> for EmulatorError {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value.to_string())
    }
}
impl<T> From<PoisonError<T>> for EmulatorError {
    fn from(value: PoisonError<T>) -> Self {
        Self::MutexInvalidState(value.to_string())
    }
}

impl From<SendError<KeyboardEvent>> for EmulatorError {
    fn from(value: SendError<KeyboardEvent>) -> Self {
        Self::IOError(format!(
            "Failed to communicate keyboard event to main thread: {}",
            value
        ))
    }
}

#[cfg(test)]
mod tests{
    use std::time::Duration;

    use super::DeviceConfig;

    #[test]
    fn test_device_config_all_false(){
        let device_config = DeviceConfig::new(false, false, false, 800);
        assert_eq!(false,device_config.is_new_chip8());
        assert_eq!(false,device_config.should_halt_on_invalid());
        assert!(device_config.get_throttling_config().is_none());
    }
    #[test]
    fn test_device_config_throttling_enabled_and_new_chip8(){
        let device_config = DeviceConfig::new(true, false, true, 100);
        const EXPECTED_INSTRUCTION_TIME_MS:u64 = 10;
        assert_eq!(true,device_config.is_new_chip8());
        assert_eq!(false,device_config.should_halt_on_invalid());
        assert_eq!(Some(Duration::from_millis(EXPECTED_INSTRUCTION_TIME_MS)),device_config.get_throttling_config());
    }
}