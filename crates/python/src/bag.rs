use std::collections::HashMap;

use rosbags_lib::Bag as RustBag;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
pub struct Bag {
    inner: RustBag,
}

#[pymethods]
impl Bag {
    #[new]
    pub fn new<'p>(
        py: Python<'p>,
        bag_uri: &str,
    ) -> Self {

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let inner = RustBag::try_from_path(bag_uri).await.unwrap();
                Bag {
                    inner
                }
            })
    }
}
