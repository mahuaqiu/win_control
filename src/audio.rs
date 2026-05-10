use pyo3::prelude::*;

pub fn create_module(py: Python<'_>) -> PyResult<Bound<'_, PyModule>> {
    let m = PyModule::new_bound(py, "audio")?;
    // TODO: 在后续任务中实现音频控制功能
    Ok(m)
}