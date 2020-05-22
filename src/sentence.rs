use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use conllu::graph::{Node, Sentence};
use conllu::token::Token;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::class::iter::PyIterProtocol;
use pyo3::class::mapping::PyMappingProtocol;
use pyo3::class::sequence::PySequenceProtocol;
use pyo3::exceptions;
use pyo3::prelude::*;

/// Sentence that can be annotated.
#[pyclass(name=Sentence)]
pub struct PySentence {
    inner: Rc<RefCell<Sentence>>,
}

#[pymethods]
impl PySentence {
    /// Construct a new sentence from forms and (optionally) POS tags.
    ///
    /// The constructor will throw a `ValueError` if POS tags are
    /// provided, but the number or tags is not equal to the number of
    /// tokens.
    #[new]
    fn __new__(forms: Vec<&str>) -> Self {
        let sent = forms.into_iter().map(Token::new).collect::<Sentence>();

        PySentence {
            inner: Rc::new(sent.into()),
        }
    }
}

impl PySentence {
    pub fn inner(&self) -> Ref<Sentence> {
        self.inner.borrow()
    }

    pub fn inner_mut(&self) -> RefMut<Sentence> {
        self.inner.borrow_mut()
    }
}

impl From<Sentence> for PySentence {
    fn from(sentence: Sentence) -> Self {
        PySentence {
            inner: Rc::new(RefCell::new(sentence)),
        }
    }
}

#[pyproto]
impl PyIterProtocol for PySentence {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<PySentenceIterator> {
        Ok(PySentenceIterator {
            sent: slf.inner.clone(),
            idx: 0,
        })
    }
}

#[pyproto]
impl PyObjectProtocol for PySentence {
    fn __repr__(&self) -> PyResult<String> {
        let token_reprs = (0..self.inner.borrow().len())
            .map(|token_idx| {
                PyToken {
                    token_idx,
                    sent: self.inner.clone(),
                }
                .__repr__()
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(format!("Sentence([{}])", token_reprs.join(", ")))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.inner.borrow()))
    }
}

#[pyproto]
impl PySequenceProtocol for PySentence {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.borrow().len())
    }

    fn __getitem__(&self, idx: isize) -> PyResult<PyToken> {
        if idx >= self.inner.borrow().len() as isize || idx < 0 {
            Err(exceptions::IndexError::py_err("token index out of range"))
        } else {
            Ok(PyToken {
                sent: self.inner.clone(),
                token_idx: idx as usize,
            })
        }
    }
}

/// Iterator over the nodes in a dependency graph.
///
/// The nodes are returned in sentence-linear order.
#[pyclass(name=SentenceIterator)]
pub struct PySentenceIterator {
    sent: Rc<RefCell<Sentence>>,
    idx: usize,
}

#[pyproto]
impl PyIterProtocol for PySentenceIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyResult<Py<PySentenceIterator>> {
        Ok(slf.into())
    }

    fn __next__(mut slf: PyRefMut<Self>) -> PyResult<Option<PyToken>> {
        let slf = &mut *slf;

        if slf.idx < slf.sent.borrow().len() {
            let token = PyToken {
                sent: slf.sent.clone(),
                token_idx: slf.idx,
            };

            slf.idx += 1;

            Ok(Some(token))
        } else {
            Ok(None)
        }
    }
}

/// Token that can be annotated.
#[pyclass(name=Token)]
pub struct PyToken {
    sent: Rc<RefCell<Sentence>>,
    token_idx: usize,
}

#[pymethods]
impl PyToken {
    /// Get the morphological features of a token.
    #[getter]
    fn get_features(&self) -> PyFeatures {
        PyFeatures {
            sent: self.sent.clone(),
            token_idx: self.token_idx,
        }
    }

    /// Get the form of the token.
    #[getter]
    fn get_form(&self) -> Option<String> {
        match self.sent.borrow()[self.token_idx] {
            Node::Token(ref token) => Some(token.form().to_owned()),
            Node::Root => None,
        }
    }

    /// Get the (dependency) head.
    #[getter]
    fn get_head(&self) -> Option<usize> {
        self.sent
            .borrow()
            .dep_graph()
            .head(self.token_idx)
            .map(|triple| triple.head())
    }

    /// Get the (dependency) head relation.
    #[getter]
    fn get_head_rel(&self) -> Option<String> {
        self.sent
            .borrow()
            .dep_graph()
            .head(self.token_idx)
            .and_then(|triple| triple.relation().map(ToOwned::to_owned))
    }

    /// Get the lemma.
    #[getter]
    fn get_lemma(&self) -> Option<String> {
        match self.sent.borrow()[self.token_idx] {
            Node::Token(ref token) => token.lemma().map(ToOwned::to_owned),
            Node::Root => None,
        }
    }

    /// Get miscellaneous features of a token.
    #[getter]
    fn get_misc(&self) -> PyMisc {
        PyMisc {
            sent: self.sent.clone(),
            token_idx: self.token_idx,
        }
    }

    /// Get the univeral part-of-speech.
    #[getter]
    fn get_upos(&self) -> Option<String> {
        match self.sent.borrow()[self.token_idx] {
            Node::Token(ref token) => token.upos().map(ToOwned::to_owned),
            Node::Root => None,
        }
    }

    /// Get the language-specific part-of-speech.
    #[getter]
    fn get_xpos(&self) -> Option<String> {
        match self.sent.borrow()[self.token_idx] {
            Node::Token(ref token) => token.xpos().map(ToOwned::to_owned),
            Node::Root => None,
        }
    }
}

#[pyproto]
impl PyObjectProtocol for PyToken {
    fn __repr__(&self) -> PyResult<String> {
        match self.sent.borrow()[self.token_idx] {
            Node::Root => Ok("Root".to_string()),
            Node::Token(ref token) => {
                let mut attrs = Vec::new();
                attrs.push(format!("form = '{}'", token.form()));

                if let Some(upos) = token.upos() {
                    attrs.push(format!("upos = '{}'", upos));
                };

                if let Some(xpos) = token.xpos() {
                    attrs.push(format!("xpos = '{}'", xpos));
                };

                if let Some(triple) = self.sent.borrow().dep_graph().head(self.token_idx) {
                    attrs.push(format!("head = {}", triple.head()));

                    if let Some(relation) = triple.relation() {
                        attrs.push(format!("relation = {}", relation));
                    }
                }

                Ok(format!("Token({})", attrs.join(", ")))
            }
        }
    }
}

/// Morphological features.
#[pyclass(name=Features)]
pub struct PyFeatures {
    sent: Rc<RefCell<Sentence>>,
    token_idx: usize,
}

#[pymethods]
impl PyFeatures {
    fn contains(&self, name: &str) -> PyResult<bool> {
        let token = self.token()?;
        Ok(token.features().get(name).is_some())
    }
}

impl PyFeatures {
    fn token(&self) -> PyResult<Ref<Token>> {
        let sent = self.sent.borrow();

        if sent[self.token_idx].is_root() {
            return Err(exceptions::KeyError::py_err(
                "root node does not have features",
            ));
        }

        let token = Ref::map(sent, |sent| sent[self.token_idx].token().unwrap());

        Ok(token)
    }

    fn token_mut(&mut self) -> PyResult<RefMut<Token>> {
        let sent = self.sent.borrow_mut();

        if sent[self.token_idx].is_root() {
            return Err(exceptions::KeyError::py_err(
                "root node does not have features",
            ));
        }

        let token = RefMut::map(sent, |sent| sent[self.token_idx].token_mut().unwrap());

        Ok(token)
    }
}

#[pyproto]
impl PyMappingProtocol for PyFeatures {
    fn __delitem__(&mut self, name: &str) -> PyResult<()> {
        let mut token = self.token_mut()?;

        let _ = token.features_mut().remove(name).ok_or_else(|| {
            exceptions::KeyError::py_err(format!("features set does not contain feature: {}", name))
        })?;

        Ok(())
    }

    fn __getitem__(&self, name: &str) -> PyResult<String> {
        let token = self.token()?;

        token
            .features()
            .get(name)
            .map(ToOwned::to_owned)
            .ok_or_else(|| exceptions::KeyError::py_err(format!("unknown feature: {}", name)))
    }

    fn __setitem__(&mut self, name: String, value: String) -> PyResult<()> {
        let mut token = self.token_mut()?;

        token.features_mut().insert(name, value);

        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for PyFeatures {
    fn __repr__(&self) -> PyResult<String> {
        let dict_repr = match &self.sent.borrow()[self.token_idx] {
            Node::Root => String::new(),
            Node::Token(token) => {
                let fvals = token
                    .features()
                    .iter()
                    .map(|(f, v)| format!("\"{}\": \"{}\"", f, v))
                    .collect::<Vec<_>>();
                fvals.join(", ")
            }
        };

        Ok(format!("Features {{{}}}", dict_repr))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}

/// Miscellaneous features.
#[pyclass(name=Misc)]
pub struct PyMisc {
    sent: Rc<RefCell<Sentence>>,
    token_idx: usize,
}

#[pymethods]
impl PyMisc {
    fn contains(&self, name: &str) -> PyResult<bool> {
        let token = self.token()?;
        Ok(token.misc().get(name).is_some())
    }
}

impl PyMisc {
    fn token(&self) -> PyResult<Ref<Token>> {
        let sent = self.sent.borrow();

        if sent[self.token_idx].is_root() {
            return Err(exceptions::KeyError::py_err(
                "root node does not have misc features",
            ));
        }

        let token = Ref::map(sent, |sent| sent[self.token_idx].token().unwrap());

        Ok(token)
    }

    fn token_mut(&mut self) -> PyResult<RefMut<Token>> {
        let sent = self.sent.borrow_mut();

        if sent[self.token_idx].is_root() {
            return Err(exceptions::KeyError::py_err(
                "root node does not have misc features",
            ));
        }

        let token = RefMut::map(sent, |sent| sent[self.token_idx].token_mut().unwrap());

        Ok(token)
    }
}

#[pyproto]
impl PyMappingProtocol for PyMisc {
    fn __delitem__(&mut self, name: &str) -> PyResult<()> {
        let mut token = self.token_mut()?;

        let _ = token.misc_mut().remove(name).ok_or_else(|| {
            exceptions::KeyError::py_err(format!(
                "misc feature set does not contain feature: {}",
                name
            ))
        })?;

        Ok(())
    }

    fn __getitem__(&self, name: &str) -> PyResult<String> {
        let token = self.token()?;

        token
            .misc()
            .get(name)
            // On the Rust side, we can have features without values.
            // Not sure if/how we want to handle this in Python.
            .and_then(|feature| feature.as_ref())
            .map(ToOwned::to_owned)
            .ok_or_else(|| exceptions::KeyError::py_err(format!("unknown feature: {}", name)))
    }

    fn __setitem__(&mut self, name: String, value: String) -> PyResult<()> {
        let mut token = self.token_mut()?;

        token.misc_mut().insert(name, Some(value));

        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for PyMisc {
    fn __repr__(&self) -> PyResult<String> {
        let dict_repr = match &self.sent.borrow()[self.token_idx] {
            Node::Root => String::new(),
            Node::Token(token) => {
                let fvals = token
                    .misc()
                    .iter()
                    // Filter out features without values.
                    .filter_map(|(f, v)| v.as_ref().map(|v| (f, v)))
                    .map(|(f, v)| format!("\"{}\": \"{}\"", f, v))
                    .collect::<Vec<_>>();
                fvals.join(", ")
            }
        };

        Ok(format!("Misc {{{}}}", dict_repr))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }
}
