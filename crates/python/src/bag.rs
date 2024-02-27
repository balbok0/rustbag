use ros_msg::msg_value::MsgValue;
use rosbags_lib::{bag::BagMessageIterator, Bag as RustBag};
use pyo3::{exceptions::PyValueError, prelude::*};

use tokio::runtime::Runtime;

#[pyclass]
pub struct Bag {
    inner: RustBag,

    runtime: Runtime,
}

#[pymethods]
impl Bag {
    #[new]
    pub fn new<'p>(
        py: Python<'p>,
        bag_uri: &str,
    ) -> Self {

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let inner = runtime.block_on(async {
            RustBag::try_from_path(bag_uri).await.unwrap()
        });

        Self {
            inner,
            runtime,
        }
    }

    pub fn read_messages(slf: PyRef<'_, Self>, verbose: bool, topics: Option<Vec<String>>, start: Option<u64>, end: Option<u64>) -> PyResult<Py<PythonMessageIter>> {
        let bag_iter = slf.runtime.block_on(
            async {
                slf.inner.read_messages(topics, start, end, verbose).await
            }
        );
        let python_iter = PythonMessageIter {
            inner: bag_iter
        };
        Ok(Py::new(slf.py(), python_iter)?)
    }
}


#[pyclass]
pub struct PythonMessageIter {
    inner: BagMessageIterator,
}

#[pymethods]
impl PythonMessageIter {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PythonMsgValue> {
        slf.inner.next().map(PythonMsgValue::from)
    }
}

#[pyclass]
pub struct PythonMsgValue {
    inner: MsgValue,
}

impl From<MsgValue> for PythonMsgValue {
    fn from(inner: MsgValue) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PythonMsgValue {
    pub fn fields(slf: PyRef<'_, Self>) -> Vec<String> {
        slf.inner.fields()
    }

    pub fn __getattr__(slf: PyRef<'_, Self>, name: &str) -> PyResult<()> {
        slf.inner.field(name,to_string()).ok_or(PyValueError::new_err("Could not find key"))
    }
}
