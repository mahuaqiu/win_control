use pyo3::prelude::*;

mod error;
mod display;
mod audio;

use error::WinCtrlError;

#[pymodule]
fn winctrl(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("WinCtrlError", py.get_type_bound::<WinCtrlError>())?;

    // Create and register display submodule
    let display_module = PyModule::new_bound(py, "display")?;
    display::display(&display_module)?;

    m.add_submodule(&display_module)?;

    // Create and register audio submodule
    let audio_module = audio::create_module(py)?;
    m.add_submodule(&audio_module)?;

    // Register submodules in sys.modules to support 'from winctrl.display import ...'
    let sys = py.import_bound("sys")?;
    let modules = sys.getattr("modules")?;
    modules.set_item("winctrl.display", &display_module)?;
    modules.set_item("winctrl.audio", &audio_module)?;

    Ok(())
}