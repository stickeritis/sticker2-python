use std::rc::Rc;

use pyo3::exceptions;
use pyo3::prelude::*;
use sticker2::input::Tokenize;
use sticker2::tagger::Tagger;
use tch::Device;

use crate::io::Model;
use crate::{PyConfig, PySentence};

#[pyclass(name=Annotator)]
pub struct PyAnnotator {
    tagger: Rc<Tagger>,
    tokenizer: Rc<Box<dyn Tokenize>>,
}

#[pymethods]
impl PyAnnotator {
    #[new]
    fn __new__(config: &PyConfig) -> PyResult<Self> {
        let model = Model::load(&config.as_ref(), Device::Cpu).map_err(|err| {
            exceptions::IOError::py_err(format!("cannot load sticker2 model: {}", err.to_string()))
        })?;

        let tagger = Tagger::new(Device::Cpu, model.model, model.encoders);

        Ok(PyAnnotator {
            tagger: Rc::new(tagger),
            tokenizer: Rc::new(model.tokenizer),
        })
    }

    /// annotate_sentence(sentence)
    /// --
    ///
    /// Annotate a sentence. The annotated sentences are returned.
    ///
    /// Parameters
    /// ----------
    /// sentence : Sentence
    ///     Sentence object to annotate.
    fn annotate_sentence(&self, sentence: PyRef<PySentence>) -> PyResult<PySentence> {
        self.annotate_sentences(vec![sentence])
            .map(|mut s| s.pop().expect("Tagging returned empty Vec"))
    }

    /// annotate_sentences(sentences)
    /// --
    ///
    /// Annotate a list of sentences. The annotated sentences are returned.
    ///
    /// Parameters
    /// ----------
    /// sentences : list
    ///     List of Sentence objects to annotate.
    fn annotate_sentences(&self, sentences: Vec<PyRef<PySentence>>) -> PyResult<Vec<PySentence>> {
        let mut sentences_with_pieces = sentences
            .into_iter()
            .map(|sent| self.tokenizer.tokenize(sent.inner().clone()))
            .collect::<Vec<_>>();

        self.tagger
            .tag_sentences(&mut sentences_with_pieces)
            .map_err(|err| exceptions::RuntimeError::py_err(err.to_string()))?;

        Ok(sentences_with_pieces
            .into_iter()
            .map(|with_pieces| with_pieces.sentence.into())
            .collect())
    }
}
