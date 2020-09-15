use std::ops::Deref;
use std::sync::Arc;

use pyo3::exceptions;
use pyo3::prelude::*;
use sticker2::input::Tokenize;
use sticker2::tagger::Tagger;
use tch::Device;

use crate::io::Model;
use crate::{PyConfig, PySentence};

/// A wrapper of `Tagger` that is `Send + Sync`.
///
/// Tensors are not thread-safe in the general case, but
/// multi-threaded use is safe if no (in-place) modifications are
/// made:
///
/// https://discuss.pytorch.org/t/is-evaluating-the-network-thread-safe/37802
struct TaggerWrap(Tagger);

unsafe impl Send for TaggerWrap {}

unsafe impl Sync for TaggerWrap {}

impl Deref for TaggerWrap {
    type Target = Tagger;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[pyclass(name=Annotator)]
pub struct PyAnnotator {
    tagger: Arc<TaggerWrap>,
    tokenizer: Arc<Box<dyn Tokenize>>,
}

#[pymethods]
impl PyAnnotator {
    #[new]
    fn __new__(config: &PyConfig) -> PyResult<Self> {
        let model = Model::load(&config.as_ref(), Device::Cpu).map_err(|err| {
            exceptions::PyIOError::new_err(format!(
                "cannot load sticker2 model: {}",
                err.to_string()
            ))
        })?;

        let tagger = Tagger::new(Device::Cpu, model.model, model.encoders);

        Ok(PyAnnotator {
            tagger: Arc::new(TaggerWrap(tagger)),
            tokenizer: Arc::new(model.tokenizer),
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
            .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))?;

        Ok(sentences_with_pieces
            .into_iter()
            .map(|with_pieces| with_pieces.sentence.into())
            .collect())
    }
}
