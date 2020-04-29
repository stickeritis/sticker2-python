use pyo3::prelude::*;

mod sentence;
pub use sentence::{PySentence, PySentenceIterator, PyToken};

pub(crate) mod util;

#[pymodule]
fn sticker2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySentence>()
}
