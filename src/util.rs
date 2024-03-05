use std::sync::mpsc::SendError;
use std::sync::PoisonError;
use sdl2::IntegerOrSdlError;
use sdl2::video::WindowBuildError;
use crate::device::keyboard::KeyboardEvent;

pub type EmulatorResult<T> = Result<T, EmulatorError>;


#[derive(Clone, Debug)]
pub enum EmulatorError {
    SdlError(String),
    AllocationError,
    IOError(String),
    MutexInvalidState(String)
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
            IntegerOrSdlError::IntegerOverflows(x, y) => { Self::SdlError(format!("{} - {}", x, y)) }
            IntegerOrSdlError::SdlError(str) => { Self::SdlError(str) }
        }
    }
}
impl From<std::io::Error> for EmulatorError{
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value.to_string())
    }
}
impl<T> From<PoisonError<T>> for EmulatorError{
    fn from(value: PoisonError<T>) -> Self {
        Self::MutexInvalidState(value.to_string())
    }
}

impl From<SendError<KeyboardEvent>> for EmulatorError{
    fn from(value: SendError<KeyboardEvent>) -> Self {
        Self::IOError(format!("Failed to communicate keyboard stats to main thread: {}",value))
    }
}