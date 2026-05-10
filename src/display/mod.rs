use pyo3::prelude::*;

mod error;
mod monitor;

pub use error::{DisplayError, DisplayErrorInner, MonitorNotFoundError, ResolutionNotSupportedError, DisplayPermissionError};
pub use monitor::list_monitors_impl;

/// 显示器信息
#[pyclass]
#[derive(Clone)]
pub struct MonitorInfo {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    is_primary: bool,
}

impl MonitorInfo {
    pub fn new(id: String, name: String, is_primary: bool) -> Self {
        Self { id, name, is_primary }
    }
}

/// 分辨率信息
#[pyclass]
#[derive(Clone)]
pub struct Resolution {
    #[pyo3(get)]
    width: u32,
    #[pyo3(get)]
    height: u32,
    #[pyo3(get)]
    refresh_rate: u32,
}

impl Resolution {
    pub fn new(width: u32, height: u32, refresh_rate: u32) -> Self {
        Self { width, height, refresh_rate }
    }
}

/// 枚举所有显示器
#[pyfunction]
pub fn list_monitors() -> PyResult<Vec<MonitorInfo>> {
    list_monitors_impl().map_err(|e: DisplayErrorInner| DisplayError::new_err(e.to_string()))
}

#[pymodule]
pub fn display(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    m.add("DisplayError", py.get_type_bound::<DisplayError>())?;
    m.add("MonitorNotFoundError", py.get_type_bound::<MonitorNotFoundError>())?;
    m.add("ResolutionNotSupportedError", py.get_type_bound::<ResolutionNotSupportedError>())?;
    m.add("DisplayPermissionError", py.get_type_bound::<DisplayPermissionError>())?;

    m.add_class::<MonitorInfo>()?;
    m.add_class::<Resolution>()?;

    // 添加函数到模块
    m.add_function(wrap_pyfunction!(list_monitors, m)?)?;

    Ok(())
}