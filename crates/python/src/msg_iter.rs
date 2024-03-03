use rustbag::BagMessageIterator;
use pyo3::prelude::*;

use crate::types::MsgIterValue;


#[pyclass]
pub struct PythonMessageIter {
    pub(crate) inner: BagMessageIterator,
}

#[pymethods]
impl PythonMessageIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<MsgIterValue> {
        slf.inner.next()
    }
}