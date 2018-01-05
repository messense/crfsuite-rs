extern crate libc;
extern crate crfsuite_sys;

use std::{mem, ptr};
use crfsuite_sys::*;

#[derive(Debug, Clone)]
pub enum CrfSuiteError {
    Incompatible,
    InternalLogic,
    NotImplemented,
    NotSupported,
    OutOfMemory,
    Overflow,
    Unknown
}

pub type Result<T> = ::std::result::Result<T, CrfSuiteError>;

/// Tuple of attribute and its value.
#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub value: f64,
}

/// Type of an item (equivalent to an attribute vector) in a sequence
pub type Item = Vec<Attribute>;
/// Type of an item sequence (equivalent to item vector).
pub type ItemSequence = Vec<Item>;

impl Attribute {
    pub fn new<T: Into<String>>(name: T, value: f64) -> Self {
        Self {
            name: name.into(),
            value: value
        }
    }
}

/// The trainer
/// It maintains a data set for training, and provides an interface
/// to various graphical models and training algorithms.
#[derive(Debug)]
pub struct Trainer {
    data: *mut crfsuite_data_t,
    trainer: *mut crfsuite_trainer_t,
}

impl Trainer {
    pub fn new() -> Result<Self> {
        unsafe {
            let data_ptr = libc::malloc(mem::size_of::<crfsuite_data_t>()) as *mut crfsuite_data_t;
            if !data_ptr.is_null() {
                crfsuite_data_init(data_ptr);
            }
        }
        unimplemented!()
    }

    pub fn append() {
    }

    pub fn select() {
    }

    pub fn train() {
    }

    pub fn params() {
    }

    pub fn set() {
    }

    pub fn get() {
    }

    pub fn help() {
    }
}

impl Drop for Trainer {
    fn drop(&mut self) {
        unsafe {
            if !self.data.is_null() {
                libc::free(self.data as *mut _);
                self.data = ptr::null_mut();
            }
            if !self.trainer.is_null() {
                // FIXME
            }
        }
    }
}

/// The tagger
/// provides the functionality for predicting label sequences for input sequences using a model.
#[derive(Debug)]
pub struct Tagger {
    model: *mut crfsuite_model_t,
    tagger: *mut crfsuite_tagger_t,
}

impl Tagger {
    pub fn new() {
    }

    pub fn open() {
    }

    pub fn close() {
    }

    pub fn labels() {
    }

    pub fn tag() {
    }

    pub fn set() {
    }

    pub fn viterbi() {
    }

    pub fn probability() {
    }

    pub fn marginal() {
    }
}

unsafe impl Send for Tagger {}
unsafe impl Sync for Tagger {}
