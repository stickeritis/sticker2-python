use std::cell::{Ref, RefCell};
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

use pyo3::class::basic::PyObjectProtocol;
use pyo3::exceptions;
use pyo3::prelude::*;

use sticker2::config::{Config, TomlRead};

/// Config(file)
/// --
///
/// Tagger configuration.
#[pyclass(name=Config)]
pub struct PyConfig {
    inner: Rc<RefCell<Config>>,
}

impl PyConfig {
    pub fn as_ref(&self) -> Ref<Config> {
        self.inner.borrow()
    }
}

#[pymethods]
impl PyConfig {
    #[new]
    fn __new__(path: &str) -> PyResult<Self> {
        let reader = BufReader::new(File::open(path).map_err(|err| {
            exceptions::IOError::py_err(format!(
                "cannot read sticker configuration: {}",
                err.to_string()
            ))
        })?);
        let mut config = Config::from_toml_read(reader).map_err(|err| {
            exceptions::ValueError::py_err(format!(
                "cannot read parse configuration: {}",
                err.to_string()
            ))
        })?;

        config.relativize_paths(path).map_err(|err| {
            exceptions::IOError::py_err(format!("cannot relativize paths: {}", err.to_string()))
        })?;

        Ok(PyConfig {
            inner: Rc::new(RefCell::new(config)),
        })
    }

    #[getter]
    fn get_labeler(&self) -> PyLabeler {
        PyLabeler {
            config: self.inner.clone(),
        }
    }

    #[getter]
    fn get_model(&self) -> PyModel {
        PyModel {
            config: self.inner.clone(),
        }
    }
}

#[pyproto]
impl PyObjectProtocol for PyConfig {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.inner.borrow()))
    }
}

#[pyclass(name=Model)]
pub struct PyModel {
    config: Rc<RefCell<Config>>,
}

#[pymethods]
impl PyModel {
    #[getter]
    fn get_parameters(&self) -> String {
        self.config.borrow().model.parameters.to_owned()
    }

    #[getter]
    fn get_pretrain_config(&self) -> String {
        self.config.borrow().model.pretrain_config.to_owned()
    }
}

#[pyproto]
impl PyObjectProtocol for PyModel {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.config.borrow().model))
    }
}

#[pyclass(name=Labeler)]
pub struct PyLabeler {
    config: Rc<RefCell<Config>>,
}

#[pymethods]
impl PyLabeler {
    #[getter]
    fn get_labels(&self) -> String {
        self.config.borrow().labeler.labels.clone()
    }
}

#[pyproto]
impl PyObjectProtocol for PyLabeler {
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.config.borrow().labeler))
    }
}
