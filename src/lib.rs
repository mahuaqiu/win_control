use pyo3::prelude::*;

mod error;
mod display;
mod audio;

use error::WinCtrlError;

#[pymodule]
fn winctrl(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("WinCtrlError", _py.get_type_bound::<WinCtrlError>())?;
    m.add_submodule(&display::create_module(_py)?)?;
    m.add_submodule(&audio::create_module(_py)?)?;
    Ok(())
}