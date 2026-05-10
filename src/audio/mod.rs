use pyo3::prelude::*;

mod error;
mod device;
mod volume;
mod policy_config;

pub use error::{AudioError, AudioErrorInner, DeviceNotFoundError, DeviceDisabledError, AudioPermissionError};
pub use policy_config::{IPolicyConfig, IID_IPolicyConfig, CLSID_CPolicyConfigClient, create_policy_config};

/// 设备信息
#[pyclass]
#[derive(Clone)]
pub struct DeviceInfo {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    device_type: String,
    #[pyo3(get)]
    state: String,
}

impl DeviceInfo {
    pub fn new(id: String, name: String, device_type: String, state: String) -> Self {
        Self { id, name, device_type, state }
    }
}

/// 设备状态
#[pyclass]
#[derive(Clone)]
pub struct DeviceState {
    #[pyo3(get)]
    state: String,
    #[pyo3(get)]
    is_default: bool,
    #[pyo3(get)]
    volume: u32,  // 音量百分比 0-100
    #[pyo3(get)]
    is_muted: bool,
}

impl DeviceState {
    pub fn new(state: String, is_default: bool, volume: u32, is_muted: bool) -> Self {
        Self { state, is_default, volume, is_muted }
    }
}

#[pymodule]
pub fn audio(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    // 注册异常类
    m.add("AudioError", py.get_type_bound::<AudioError>())?;
    m.add("DeviceNotFoundError", py.get_type_bound::<DeviceNotFoundError>())?;
    m.add("DeviceDisabledError", py.get_type_bound::<DeviceDisabledError>())?;
    m.add("AudioPermissionError", py.get_type_bound::<AudioPermissionError>())?;

    // 注册数据类型
    m.add_class::<DeviceInfo>()?;
    m.add_class::<DeviceState>()?;

    // 注册设备枚举函数
    m.add_function(wrap_pyfunction!(device::list_devices, m)?)?;
    m.add_function(wrap_pyfunction!(device::get_device_state, m)?)?;

    // 注册设备禁用/启用函数 (Task 15)
    m.add_function(wrap_pyfunction!(device::disable_device, m)?)?;
    m.add_function(wrap_pyfunction!(device::enable_device, m)?)?;

    // 注册默认设备管理函数 (Task 16)
    m.add_function(wrap_pyfunction!(device::get_default_device, m)?)?;
    m.add_function(wrap_pyfunction!(device::set_default_device, m)?)?;

    // 注册音量控制函数
    m.add_function(wrap_pyfunction!(volume::get_volume, m)?)?;
    m.add_function(wrap_pyfunction!(volume::set_volume, m)?)?;
    m.add_function(wrap_pyfunction!(volume::get_mute, m)?)?;
    m.add_function(wrap_pyfunction!(volume::set_mute, m)?)?;

    // 注册输入设备音量控制函数
    m.add_function(wrap_pyfunction!(volume::get_input_volume, m)?)?;
    m.add_function(wrap_pyfunction!(volume::set_input_volume, m)?)?;
    m.add_function(wrap_pyfunction!(volume::get_input_mute, m)?)?;
    m.add_function(wrap_pyfunction!(volume::set_input_mute, m)?)?;

    Ok(())
}