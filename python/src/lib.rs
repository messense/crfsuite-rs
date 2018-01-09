#![feature(proc_macro, specialization, const_fn)]
extern crate pyo3;
extern crate crfsuite;

use pyo3::prelude::*;

#[py::class]
struct Model {
    inner: crfsuite::Model
}

#[py::methods]
impl Model {

    #[new]
    fn __new__(obj: &PyRawObject, path: String) -> PyResult<()> {
        obj.init(|t| Model { inner: crfsuite::Model::from_file(&path).unwrap() })
    }

    fn tag(&self, py: Python, items: Vec<Vec<(String, f64)>>) -> PyResult<Vec<String>> {
       let ret = py.allow_threads(move || {
            let mut attrs = Vec::with_capacity(items.len());
            for item in &items {
                let seq: Vec<crfsuite::Attribute> = item.iter().map(|x| crfsuite::Attribute::new(x.0.to_string(), x.1)).collect();
                attrs.push(seq);
            }
            let mut tagger = self.inner.tagger().unwrap();
            tagger.tag(&attrs).unwrap()
        });
       Ok(ret)
    }
}

/// crfsuite
#[py::modinit(_crfsuite)]
fn init_module(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Model>()?;

    Ok(())
}
