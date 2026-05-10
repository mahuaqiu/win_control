use pyo3::prelude::*;

mod error;

pub use error::{DisplayError, MonitorNotFoundError, ResolutionNotSupportedError, DisplayPermissionError};

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

pub fn create_module(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
    let m = PyModule::new_bound(py, "display")?;

    m.add("DisplayError", py.get_type_bound::<DisplayError>())?;
    m.add("MonitorNotFoundError", py.get_type_bound::<MonitorNotFoundError>())?;
    m.add("ResolutionNotSupportedError", py.get_type_bound::<ResolutionNotSupportedError>())?;
    m.add("DisplayPermissionError", py.get_type_bound::<DisplayPermissionError>())?;

    m.add_class::<MonitorInfo>()?;
    m.add_class::<Resolution>()?;

    Ok(m)
}