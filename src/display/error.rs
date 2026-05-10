use pyo3::create_exception;
use pyo3::exceptions::PyException;
use std::fmt;

/// 内部错误类型用于 Rust 代码
#[derive(Debug)]
pub enum DisplayErrorInner {
    EnumerationFailed(String),
    DeviceNotFound(String),
}

impl fmt::Display for DisplayErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DisplayErrorInner::EnumerationFailed(msg) => write!(f, "显示器枚举失败: {}", msg),
            DisplayErrorInner::DeviceNotFound(msg) => write!(f, "设备未找到: {}", msg),
        }
    }
}

impl std::error::Error for DisplayErrorInner {}

// PyO3 异常类
create_exception!(display, DisplayError, PyException);
create_exception!(display, MonitorNotFoundError, DisplayError);
create_exception!(display, ResolutionNotSupportedError, DisplayError);
create_exception!(display, DisplayPermissionError, DisplayError);