use pyo3::prelude::*;

mod sentence;
pub use sentence::{PySentence, PySentenceIterator, PyToken};

#[pymodule]
fn sticker2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySentence>()
}
