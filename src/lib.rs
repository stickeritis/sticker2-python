use pyo3::prelude::*;

mod config;
pub use config::{PyConfig, PyLabeler, PyModel};

pub(crate) mod io;

mod sentence;
pub use sentence::{PySentence, PySentenceIterator, PyToken};

mod annotator;
pub use annotator::PyAnnotator;

#[pymodule]
fn sticker2(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyAnnotator>()?;
    m.add_class::<PyConfig>()?;
    m.add_class::<PyLabeler>()?;
    m.add_class::<PyModel>()?;
    m.add_class::<PySentence>()?;

    Ok(())
}
