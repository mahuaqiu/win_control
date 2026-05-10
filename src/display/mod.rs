use pyo3::prelude::*;

mod error;

pub use error::{DisplayError, MonitorNotFoundError, ResolutionNotSupportedError, DisplayPermissionError};

pub fn create_module(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
    let m = PyModule::new_bound(py, "display")?;

    m.add("DisplayError", py.get_type_bound::<DisplayError>())?;
    m.add("MonitorNotFoundError", py.get_type_bound::<MonitorNotFoundError>())?;
    m.add("ResolutionNotSupportedError", py.get_type_bound::<ResolutionNotSupportedError>())?;
    m.add("DisplayPermissionError", py.get_type_bound::<DisplayPermissionError>())?;

    Ok(m)
}