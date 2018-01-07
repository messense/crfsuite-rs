extern crate libc;
extern crate crfsuite_sys;

use std::{mem, ptr, fmt, error, slice};
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
    ParamNotFound(String),
}

impl error::Error for CrfError {
    fn description(&self) -> &str {
        match *self {
            CrfError::CrfSuiteError(ref err) => err.description(),
            CrfError::ParamNotFound(_) => "Parameter not found"
        }
    }
}

impl fmt::Display for CrfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CrfError::CrfSuiteError(ref err) => err.fmt(f),
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

impl Attribute {
    pub fn new<T: Into<String>>(name: T, value: f64) -> Self {
        Self {
            name: name.into(),
            value: value
        }
    }
}

/// The training algorithm
#[derive(Debug, Clone)]
pub enum Algorithm {
    /// Gradient descent using the L-BFGS method
    LBFGS,
    /// Stochastic Gradient Descent with L2 regularization term
    L2SGD,
    /// Averaged Perceptron
    AP,
    /// Passive Aggressive
    PA,
    /// Adaptive Regularization Of Weight Vector
    AROW,
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match *self {
            Algorithm::LBFGS => "lbfgs",
            Algorithm::L2SGD => "l2sgd",
            Algorithm::AP => "ap",
            Algorithm::PA => "pa",
            Algorithm::AROW => "arow",
        };
        write!(f, "{}", desc)
    }
}

/// The graphical model
#[derive(Debug, Clone)]
pub enum GraphicalModel {
    /// The 1st-order Markov CRF with state and transition features (dyad features).
    /// State features are conditioned on combinations of attributes and labels,
    /// and transition features are conditioned on label bigrams.
    CRF1D,
}

impl fmt::Display for GraphicalModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match *self {
            GraphicalModel::CRF1D => "crf1d",
        };
        write!(f, "{}", desc)
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
                    return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
                }
            }
            if (*self.data).labels.is_null() {
                let ret = crfsuite_create_instance("dictionary".as_ptr() as *const _, (*self.data).labels as *mut _);
                if ret != 0 {
                    return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
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
    pub fn append(&mut self, xseq: &[Item], yseq: &[Item], group: u32) -> Result<()> {
        unimplemented!()
    }

    /// Initialize the training algorithm.
    pub fn select(&mut self, algorithm: Algorithm, typ: GraphicalModel) -> Result<bool> {
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

/// The model
#[derive(Debug)]
pub struct Model {
    model: *mut crfsuite_model_t,
    initialized: bool
}

/// The tagger
/// provides the functionality for predicting label sequences for input sequences using a model.
#[derive(Debug)]
pub struct Tagger<'a> {
    model: &'a Model,
    tagger: *mut crfsuite_tagger_t,
}

impl Model {
    fn new() -> Self {
        unsafe {
            Model {
                model: mem::uninitialized(),
                initialized: false,
            }
        }
    }

    /// Open a model file
    pub fn from_file(name: &str) -> Result<Self> {
        let mut model = Model::new();
        model.open(name)?;
        Ok(model)
    }

    /// Open a model file
    fn open(&mut self, name: &str) -> Result<()> {
        let name_cstr = CString::new(name).unwrap();
        unsafe {
            let ret = crfsuite_create_instance_from_file(name_cstr.as_ptr(), mem::transmute(&mut self.model));
            if ret != 0 {
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            self.initialized = true;
        }
        Ok(())
    }

    /// Close the model
    fn close(&mut self) {
        unsafe {
            if self.initialized {
                (*self.model).release.map(|f| f(self.model));
                self.initialized = false;
            }
        }
    }

    pub fn tagger<'a>(&'a self) -> Result<Tagger<'a>> {
        unsafe {
            let mut tagger = ptr::null_mut();
            let ret = (*self.model).get_tagger.map(|f| f(self.model, &mut tagger)).unwrap();
            if ret != 0 {
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            Ok(Tagger {
                model: self,
                tagger: tagger
            })
        }
    }

    unsafe fn get_attrs(&self) -> Result<*mut crfsuite_dictionary_t> {
        let mut attrs: *mut crfsuite_dictionary_t = ptr::null_mut();
        let ret = (*self.model).get_attrs.map(|f| f(self.model, &mut attrs)).unwrap();
        if ret != 0 {
            return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
        }
        Ok(attrs)
    }

    unsafe fn get_labels(&self) -> Result<*mut crfsuite_dictionary_t> {
        let mut labels: *mut crfsuite_dictionary_t = ptr::null_mut();
        let ret = (*self.model).get_labels.map(|f| f(self.model, &mut labels)).unwrap();
        if ret != 0 {
            return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
        }
        Ok(labels)
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.close();
    }
}

unsafe impl Send for Model {}
unsafe impl Sync for Model {}

impl<'a> Drop for Tagger<'a> {
    fn drop(&mut self) {
        unsafe { (*self.tagger).release.map(|f| f(self.tagger)); }
    }
}

impl<'a> Tagger<'a> {
    /// Obtain the list of labels
    pub fn labels(&self) -> Result<Vec<String>> {
        unsafe {
            let labels = self.model.get_labels()?;
            let length = (*labels).num.map(|f| f(labels)).unwrap();
            let mut lseq = Vec::with_capacity(length as usize);
            for i in 0..length {
                let mut label: *mut libc::c_char = mem::uninitialized();
                let ret = (*labels).to_string.map(|f| f(labels, i, mem::transmute(&mut label))).unwrap();
                if ret != 0 {
                    (*labels).release.map(|f| f(labels));
                    return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
                }
                lseq.push(CStr::from_ptr(label).to_string_lossy().into_owned());
                (*labels).free.map(|f| f(labels, label));
            }
            (*labels).release.map(|f| f(labels));
            Ok(lseq)
        }
    }

    /// Predict the label sequence for the item sequence.
    pub fn tag(&mut self, xseq: &[Item]) -> Result<Vec<String>> {
        self.set(xseq)?;
        self.viterbi()
    }

    /// Set an item sequence.
    fn set(&mut self, xseq: &[Item]) -> Result<()> {
        unsafe {
            let mut instance: crfsuite_instance_t = mem::uninitialized();
            let attrs = self.model.get_attrs()?;
            let xseq_len = xseq.len();
            crfsuite_instance_init_n(&mut instance, xseq_len as i32);
            let crf_items = slice::from_raw_parts_mut(instance.items, instance.num_items as usize);
            for t in 0..xseq_len {
                let items = &xseq[t];
                let mut crf_item = &mut crf_items[t];
                // Set the attributes in the item
                crfsuite_item_init(crf_item);
                for attr in items.iter() {
                    let name_cstr = CString::new(&attr.name[..]).unwrap();
                    let aid = (*attrs).to_id.map(|f| f(attrs, name_cstr.as_ptr())).unwrap();
                    if aid >= 0 {
                        let mut cont: crfsuite_attribute_t = mem::uninitialized();
                        crfsuite_attribute_set(&mut cont, aid, attr.value);
                        crfsuite_item_append_attribute(crf_item, &cont);
                    }
                }
            }

            // Set the instance to the tagger
            let ret = (*self.tagger).set.map(|f| f(self.tagger, &mut instance)).unwrap();
            if ret != 0 {
                (*attrs).release.map(|f| f(attrs));
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            crfsuite_instance_finish(&mut instance);
            (*attrs).release.map(|f| f(attrs));
        }
        Ok(())
    }

    /// Find the Viterbi label sequence for the item sequence.
    pub fn viterbi(&self) -> Result<Vec<String>> {
        unsafe {
            // Make sure that the current instance is not empty
            let length = (*self.tagger).length.map(|f| f(self.tagger)).unwrap();
            if length <= 0 {
                return Ok(Vec::new());
            }
            let labels = self.model.get_labels()?;
            // Run the Viterbi algorithm
            let mut score: floatval_t = mem::uninitialized();
            let mut paths = Vec::with_capacity(length as usize);
            let ret = (*self.tagger).viterbi.map(|f| f(self.tagger, paths.as_mut_ptr(), &mut score)).unwrap();
            if ret != 0 {
                (*labels).release.map(|f| f(labels));
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            let mut yseq = Vec::with_capacity(length as usize);
            // Convert the Viterbi path to a label sequence
            for path in paths.into_iter() {
                let mut label: *mut libc::c_char = mem::uninitialized();
                let ret = (*labels).to_string.map(|f| f(labels, path, mem::transmute(&mut label))).unwrap();
                if ret != 0 {
                    (*labels).release.map(|f| f(labels));
                    return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
                }
                yseq.push(CStr::from_ptr(label).to_string_lossy().into_owned());
                (*labels).free.map(|f| f(labels, label));
            }
            (*labels).release.map(|f| f(labels));
            Ok(yseq)
        }
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
