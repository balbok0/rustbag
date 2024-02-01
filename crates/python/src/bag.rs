use std::collections::HashMap;

use rosbags_lib::Bag as RustBag;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
struct Bag {
    inner: RustBag,
}

impl Bag {
    // #[pyo3(signature = (bag_uri, storage_options = None))]

    // #[pyfunction]
    // pub fn new<'p>(
    //     py: Python<'p>,
    //     bag_uri: &str,
    //     storage_options: Option<HashMap<String, String>>,
    // ) -> PyResult<&'p PyAny> {

    //     pyo3_asyncio::tokio::future_into_py(py, async move {
    //         let inner = RustBag::try_from_path(bag_uri).await?;
    //         Ok(Bag {
    //             inner
    //         })
    //     })
    // }
}