mod bag;


use pyo3::prelude::*;

#[pymodule]
fn rosbags_rs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<bag::Bag>()?;
    Ok(())
}