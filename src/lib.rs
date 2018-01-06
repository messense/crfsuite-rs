extern crate libc;
extern crate crfsuite_sys;

use std::{mem, ptr, fmt, error};
use std::ffi::{CStr, CString};
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

impl error::Error for CrfSuiteError {
    fn description(&self) -> &str {
        match *self {
            CrfSuiteError::Incompatible => "Incompatible data",
            CrfSuiteError::InternalLogic => "Internal error",
            CrfSuiteError::NotImplemented => "Not implemented",
            CrfSuiteError::NotSupported => "Unsupported operation",
            CrfSuiteError::OutOfMemory => "Insufficient memory",
            CrfSuiteError::Overflow => "Overflow",
            CrfSuiteError::Unknown => "Unknown error occurred",
        }
    }
}

impl fmt::Display for CrfSuiteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match *self {
            CrfSuiteError::Incompatible => "Incompatible data",
            CrfSuiteError::InternalLogic => "Internal error",
            CrfSuiteError::NotImplemented => "Not implemented",
            CrfSuiteError::NotSupported => "Unsupported operation",
            CrfSuiteError::OutOfMemory => "Insufficient memory",
            CrfSuiteError::Overflow => "Overflow",
            CrfSuiteError::Unknown => "Unknown error occurred",
        };
        write!(f, "{}", desc)
    }
}

impl From<libc::c_int> for CrfSuiteError {
    fn from(code: libc::c_int) -> Self {
        match code {
            CRFSUITEERR_INCOMPATIBLE => CrfSuiteError::Incompatible,
            CRFSUITEERR_INTERNAL_LOGIC => CrfSuiteError::InternalLogic,
            CRFSUITEERR_NOTIMPLEMENTED => CrfSuiteError::NotImplemented,
            CRFSUITEERR_NOTSUPPORTED => CrfSuiteError::NotSupported,
            CRFSUITEERR_OUTOFMEMORY => CrfSuiteError::OutOfMemory,
            CRFSUITEERR_OVERFLOW => CrfSuiteError::Overflow,
            CRFSUITEERR_UNKNOWN => CrfSuiteError::Unknown,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CrfError {
    CrfSuiteError(CrfSuiteError),
    CreateInstanceError(String),
    ParamNotFound(String),
}

impl error::Error for CrfError {
    fn description(&self) -> &str {
        match *self {
            CrfError::CrfSuiteError(ref err) => err.description(),
            CrfError::CreateInstanceError(ref err) => err,
            CrfError::ParamNotFound(_) => "Parameter not found"
        }
    }
}

impl fmt::Display for CrfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CrfError::CrfSuiteError(ref err) => err.fmt(f),
            CrfError::CreateInstanceError(ref err) => err.fmt(f),
            CrfError::ParamNotFound(ref name) => write!(f, "Parameter {} not found", name),
        }
    }
}

pub type Result<T> = ::std::result::Result<T, CrfError>;

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
    /// Construct a trainer
    pub fn new() -> Self {
        unsafe {
            let data_ptr = libc::malloc(mem::size_of::<crfsuite_data_t>()) as *mut crfsuite_data_t;
            if !data_ptr.is_null() {
                crfsuite_data_init(data_ptr);
            }
            Self {
                data: data_ptr,
                trainer: ptr::null_mut()
            }
        }
    }

    pub fn init(&mut self) -> Result<()> {
        unsafe {
            if (*self.data).attrs.is_null() {
                let ret = crfsuite_create_instance("dictionary".as_ptr() as *const _, (*self.data).attrs as *mut _);
                if ret != 0 {
                    return Err(CrfError::CreateInstanceError("Failed to create a dictionary instance for attributes.".to_string()));
                }
            }
            if (*self.data).labels.is_null() {
                let ret = crfsuite_create_instance("dictionary".as_ptr() as *const _, (*self.data).labels as *mut _);
                if ret != 0 {
                    return Err(CrfError::CreateInstanceError("Failed to create a dictionary instance for labels.".to_string()));
                }
            }
        }
        Ok(())
    }

    /// Remove all instances in the data set
    pub fn clear(&mut self) -> Result<()> {
        if self.data.is_null() {
            return Ok(());
        }
        unsafe {
            if (*self.data).attrs.is_null() {
                (*(*self.data).attrs).release.map(|release| release((*self.data).attrs));
                (*self.data).attrs = ptr::null_mut();
            }
            if (*self.data).labels.is_null() {
                (*(*self.data).labels).release.map(|release| release((*self.data).labels));
                (*self.data).labels = ptr::null_mut();
            }
            crfsuite_data_finish(self.data);
            crfsuite_data_init(self.data);
        }
        Ok(())
    }

    /// Append an instance (item/label sequence) to the data set.
    pub fn append(&mut self, xseq: &ItemSequence, yseq: &ItemSequence, group: u32) -> Result<()> {
        unimplemented!()
    }

    /// Initialize the training algorithm.
    pub fn select(&mut self, algorithm: &str, typ: &str) -> Result<bool> {
        unimplemented!()
    }

    /// Run the training algorithm.
    ///
    /// This function starts the training algorithm with the data set given
    /// by `append()` function.
    pub fn train(&mut self, model: &str, holdout: u32) -> Result<()> {
        unimplemented!()
    }

    /// Obtain the list of parameters.
    ///
    /// This function returns the list of parameter names available for the
    /// graphical model and training algorithm specified by `select()` function.
    pub fn params(&self) -> Vec<String> {
        unsafe {
            let pms = (*self.trainer).params.map(|f| f(self.trainer)).unwrap();
            let n = (*pms).num.map(|f| f(pms)).unwrap();
            let mut ret = Vec::with_capacity(n as usize);
            for i in 0..n {
                let mut name: *mut libc::c_char = ptr::null_mut();
                (*pms).name.map(|f| f(pms, i, &mut name));
                let c_str = CStr::from_ptr(name);
                ret.push(c_str.to_string_lossy().into_owned());
            }
            ret
        }
    }

    /// Set a training parameter.
    ///
    /// This function sets a parameter value for the graphical model and
    /// training algorithm specified by `select()` function.
    pub fn set(&mut self, name: &str, value: &str) -> Result<()> {
        let name_cstr = CString::new(name).unwrap();
        let value_cstr = CString::new(value).unwrap();
        unsafe {
            let pms = (*self.trainer).params.map(|f| f(self.trainer)).unwrap();
            if (*pms).set.map(|f| f(pms, name_cstr.as_ptr(), value_cstr.as_ptr())).unwrap() != 0 {
                (*pms).release.map(|f| f(pms));
                return Err(CrfError::ParamNotFound(name.to_string()));
            }
            (*pms).release.map(|f| f(pms));
        }
        Ok(())
    }

    /// Get the value of a training parameter.
    ///
    /// This function gets a parameter value for the graphical model and
    /// training algorithm specified by `select()` function.
    pub fn get(&self, name: &str) -> Result<String> {
        let name_cstr = CString::new(name).unwrap();
        let value;
        unsafe {
            let mut value_ptr: *mut libc::c_char = ptr::null_mut();
            let pms = (*self.trainer).params.map(|f| f(self.trainer)).unwrap();
            if (*pms).get.map(|f| f(pms, name_cstr.as_ptr(), &mut value_ptr)).unwrap() != 0 {
                (*pms).release.map(|f| f(pms));
                return Err(CrfError::ParamNotFound(name.to_string()));
            }
            value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();
            (*pms).free.map(|f| f(pms, value_ptr));
            (*pms).release.map(|f| f(pms));
        }
        Ok(value)
    }

    /// Get the description of a training parameter.
    ///
    /// This function obtains the help message for the parameter specified
    /// by the name. The graphical model and training algorithm must be
    /// selected by `select()` function before calling this function.
    pub fn help(&self, name: &str) -> Result<String> {
        let name_cstr = CString::new(name).unwrap();
        let value;
        unsafe {
            let mut value_ptr: *mut libc::c_char = ptr::null_mut();
            let pms = (*self.trainer).params.map(|f| f(self.trainer)).unwrap();
            if (*pms).help.map(|f| f(pms, name_cstr.as_ptr(), ptr::null_mut(), &mut value_ptr)).unwrap() != 0 {
                (*pms).release.map(|f| f(pms));
                return Err(CrfError::ParamNotFound(name.to_string()));
            }
            value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();
            (*pms).free.map(|f| f(pms, value_ptr));
            (*pms).release.map(|f| f(pms));
        }
        Ok(value)
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
    /// Construct a tagger
    pub fn new() -> Self {
        Self {
            model: ptr::null_mut(),
            tagger: ptr::null_mut(),
        }
    }

    /// Open a model file
    pub fn open(&mut self, name: &str) -> Result<()> {
        // Close the model if it is already opened
        self.close();
        let name_cstr = CString::new(name).unwrap();
        unsafe {
            if crfsuite_create_instance_from_file(name_cstr.as_ptr(), self.model as *mut *mut _) != 0 {
                return Err(CrfError::CreateInstanceError("Failed to a model instance for tagger.".to_string()));
            }
            if (*self.model).get_tagger.map(|f| f(self.model, self.tagger as *mut *mut _)).unwrap() != 0 {
                return Err(CrfError::CreateInstanceError("Failed to obtain the tagger interface.".to_string()));
            }
        }
        Ok(())
    }

    /// Close the model
    pub fn close(&mut self) {
        unsafe {
            if !self.tagger.is_null() {
                (*self.tagger).release.map(|f| f(self.tagger));
                self.tagger = ptr::null_mut();
            }
            if !self.model.is_null() {
                (*self.model).release.map(|f| f(self.model));
                self.model = ptr::null_mut();
            }
        }
    }

    /// Obtain the list of labels
    pub fn labels(&self) -> Vec<String> {
        unimplemented!()
    }

    /// Predict the label sequence for the item sequence.
    pub fn tag(&self, xseq: &ItemSequence) -> Vec<String> {
        unimplemented!()
    }

    /// Set an item sequence.
    pub fn set(&mut self, xseq: &ItemSequence) {
    }

    /// Find the Viterbi label sequence for the item sequence.
    pub fn viterbi(&self) -> Vec<String> {
        unimplemented!()
    }

    /// Compute the probability of the label sequence.
    pub fn probability(&self, yseq: &[String]) -> f64 {
        unimplemented!()
    }

    /// Compute the marginal probability of the label.
    pub fn marginal(&self, label: &str, position: u32) -> f64 {
        unimplemented!()
    }
}

impl Drop for Tagger {
    fn drop(&mut self) {
        self.close();
    }
}

unsafe impl Send for Tagger {}
unsafe impl Sync for Tagger {}
