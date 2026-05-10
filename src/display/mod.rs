use pyo3::prelude::*;

mod error;
mod monitor;

pub use error::{DisplayError, DisplayErrorInner, MonitorNotFoundError, ResolutionNotSupportedError, DisplayPermissionError};
pub use monitor::{MonitorInfo, Resolution, list_monitors, get_current_resolution, get_all_resolutions, get_supported_resolutions, set_resolution, set_resolution_with_refresh, restore_resolution};

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
    m.add_function(wrap_pyfunction!(get_current_resolution, m)?)?;
    m.add_function(wrap_pyfunction!(get_all_resolutions, m)?)?;
    m.add_function(wrap_pyfunction!(get_supported_resolutions, m)?)?;
    m.add_function(wrap_pyfunction!(set_resolution, m)?)?;
    m.add_function(wrap_pyfunction!(set_resolution_with_refresh, m)?)?;
    m.add_function(wrap_pyfunction!(restore_resolution, m)?)?;

    Ok(())
}