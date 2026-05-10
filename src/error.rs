use pyo3::prelude::*;
use pyo3::exceptions::PyException;

// 自定义错误类型
#[derive(Debug)]
pub enum WinctrlError {
    DisplayError(String),
    AudioError(String),
    SystemError(String),
}

impl std::fmt::Display for WinctrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WinctrlError::DisplayError(msg) => write!(f, "Display error: {}", msg),
            WinctrlError::AudioError(msg) => write!(f, "Audio error: {}", msg),
            WinctrlError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl From<WinctrlError> for PyErr {
    fn from(err: WinctrlError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}