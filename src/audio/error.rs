use pyo3::create_exception;
use pyo3::exceptions::PyException;
use std::fmt;

/// 内部错误类型用于 Rust 代码
#[derive(Debug)]
pub enum AudioErrorInner {
    DeviceNotFound(String),
    EnumerationFailed(String),
    ComError(String),
    PermissionDenied(String),
    DeviceDisabled(String),
    InvalidDeviceId(String),
    VolumeError(String),
}

impl fmt::Display for AudioErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioErrorInner::DeviceNotFound(msg) => write!(f, "设备未找到: {}", msg),
            AudioErrorInner::EnumerationFailed(msg) => write!(f, "枚举失败: {}", msg),
            AudioErrorInner::ComError(msg) => write!(f, "COM错误: {}", msg),
            AudioErrorInner::PermissionDenied(msg) => write!(f, "权限被拒绝: {}", msg),
            AudioErrorInner::DeviceDisabled(msg) => write!(f, "设备已禁用: {}", msg),
            AudioErrorInner::InvalidDeviceId(msg) => write!(f, "无效的设备ID: {}", msg),
            AudioErrorInner::VolumeError(msg) => write!(f, "音量操作失败: {}", msg),
        }
    }
}

impl std::error::Error for AudioErrorInner {}

// PyO3 异常类
create_exception!(audio, AudioError, PyException);
create_exception!(audio, DeviceNotFoundError, AudioError);
create_exception!(audio, DeviceDisabledError, AudioError);
create_exception!(audio, AudioPermissionError, AudioError);