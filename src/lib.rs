use pyo3::prelude::*;

mod config;
pub use config::{PyConfig, PyLabeler, PyModel};

mod sentence;
pub use sentence::{PySentence, PySentenceIterator, PyToken};

#[pymodule]
fn sticker2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyConfig>()?;
    m.add_class::<PyLabeler>()?;
    m.add_class::<PyModel>()?;
    m.add_class::<PySentence>()
}
