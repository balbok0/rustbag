mod bag;
mod msg_iter;
mod types;


use pyo3::prelude::*;

#[pymodule]
fn rustbag(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<bag::Bag>()?;
    Ok(())
}