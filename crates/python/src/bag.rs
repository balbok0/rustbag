use std::collections::HashMap;

use rosbags_lib::Bag as RustBag;
use pyo3::prelude::*;

use tokio::runtime::Runtime;
use url::Url;

use crate::msg_iter::PythonMessageIter;


#[pyclass]
pub struct Bag {
    inner: RustBag,

    runtime: Runtime,
}

#[pymethods]
impl Bag {
    #[new]
    pub fn new<'p>(
        _py: Python<'p>,
        bag_uri: &str,
        options: Option<HashMap<&str, String>>
    ) -> Self {

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let inner = runtime.block_on(async {
            RustBag::try_new_from_url(&Url::parse(bag_uri).unwrap(), options).await.unwrap()
        });

        Self {
            inner,
            runtime,
        }
    }

    pub fn read_messages(slf: PyRef<'_, Self>, topics: Option<Vec<String>>, start: Option<u64>, end: Option<u64>) -> PyResult<Py<PythonMessageIter>> {
        let bag_iter = slf.runtime.block_on(
            async {
                slf.inner.read_messages(topics, start, end).await
            }
        );
        let python_iter = PythonMessageIter {
            inner: bag_iter
        };
        Ok(Py::new(slf.py(), python_iter)?)
    }

    pub fn num_messages(slf: PyRef<'_, Self>) -> u64 {
        slf.runtime.block_on(
            async {
                slf.inner.num_messages().await
            }
        )
    }
}