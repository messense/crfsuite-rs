#![allow(unknown_lints)]
#![allow(clippy::useless_transmute)]
#![allow(clippy::transmute_ptr_to_ref)]
#![allow(clippy::transmute_ptr_to_ptr)]
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
#[cfg(unix)]
use std::os::unix::io::{IntoRawFd, RawFd};
#[cfg(windows)]
use std::os::windows::io::{IntoRawHandle, RawHandle};
use std::path::Path;
use std::{error, fmt, mem, ptr, slice};

use crfsuite_sys::*;
#[cfg(not(windows))]
use libc::{c_char, c_int, c_uint};
use libc::{c_void, fclose, fdopen};

/// Errors from crfsuite ffi functions
#[derive(Debug, Clone, PartialEq)]
pub enum CrfSuiteError {
    /// Incompatible data
    Incompatible,
    /// Internal error
    InternalLogic,
    /// Not implemented
    NotImplemented,
    /// Unsupported operation
    NotSupported,
    /// Insufficient memory
    OutOfMemory,
    /// Overflow
    Overflow,
    /// Unknown error occurred
    Unknown,
}

impl error::Error for CrfSuiteError {}

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

#[derive(Debug, Clone, PartialEq)]
pub enum CrfError {
    /// Errors from crfsuite ffi functions
    CrfSuiteError(CrfSuiteError),
    /// Create instance error
    CreateInstanceError(String),
    /// Parameter not found
    ParamNotFound(String),
    /// Trainer algorithm not selected
    AlgorithmNotSelected,
    /// Trainer data is empty
    EmptyData,
    /// Invalid argument
    InvalidArgument(String),
    /// Invalid value
    ValueError(String),
    /// Invalid model
    InvalidModel(String),
}

impl error::Error for CrfError {}

impl fmt::Display for CrfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CrfError::CrfSuiteError(ref err) => err.fmt(f),
            CrfError::ParamNotFound(ref name) => write!(f, "Parameter {} not found", name),
            CrfError::AlgorithmNotSelected => write!(
                f,
                "The trainer is not initialized. Call Trainer::select before Trainer::train."
            ),
            CrfError::EmptyData => write!(
                f,
                "The data is empty. Call Trainer::append before Trainer::train."
            ),
            CrfError::CreateInstanceError(ref err)
            | CrfError::InvalidArgument(ref err)
            | CrfError::ValueError(ref err)
            | CrfError::InvalidModel(ref err) => err.fmt(f),
        }
    }
}

pub type Result<T> = ::std::result::Result<T, CrfError>;

/// Tuple of attribute and its value.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    /// Attribute name
    pub name: String,
    /// Attribute value
    pub value: f64,
}

/// Type of an item (equivalent to an attribute vector) in a sequence
pub type Item = Vec<Attribute>;

impl Attribute {
    #[inline]
    pub fn new<T: Into<String>>(name: T, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

impl From<String> for Attribute {
    #[inline]
    fn from(t: String) -> Self {
        Self {
            name: t,
            value: 1.0,
        }
    }
}

impl<'a> From<&'a str> for Attribute {
    #[inline]
    fn from(t: &'a str) -> Self {
        Self {
            name: t.to_string(),
            value: 1.0,
        }
    }
}

impl<T: Into<String>> From<(T, f64)> for Attribute {
    #[inline]
    fn from(t: (T, f64)) -> Self {
        let (name, value) = t;
        Self {
            name: name.into(),
            value,
        }
    }
}

/// The training algorithm
#[derive(Debug, Clone, Copy, PartialEq)]
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
            Algorithm::AP => "averaged-perceptron",
            Algorithm::PA => "passive-aggressive",
            Algorithm::AROW => "arow",
        };
        write!(f, "{}", desc)
    }
}

impl ::std::str::FromStr for Algorithm {
    type Err = CrfError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "lbfgs" => Ok(Algorithm::LBFGS),
            "l2sgd" => Ok(Algorithm::L2SGD),
            "ap" | "averaged-perceptron" => Ok(Algorithm::AP),
            "pa" | "passive-aggressive" => Ok(Algorithm::PA),
            "arow" => Ok(Algorithm::AROW),
            _ => Err(CrfError::InvalidArgument(s.to_string())),
        }
    }
}

/// The graphical model
#[derive(Debug, Clone, Copy, PartialEq)]
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

impl ::std::str::FromStr for GraphicalModel {
    type Err = CrfError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "1d" | "crf1d" => Ok(GraphicalModel::CRF1D),
            _ => Err(CrfError::InvalidArgument(s.to_string())),
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
    verbose: bool,
}

impl Default for Trainer {
    fn default() -> Self {
        Trainer::new(false)
    }
}

#[cfg(not(windows))]
extern "C" {
    fn vsnprintf(buf: *mut c_char, size: c_uint, fmt: *const c_char, va_list: *mut c_void);
}

#[cfg(not(windows))]
extern "C" fn logging_callback(
    user: *mut c_void,
    format: *const c_char,
    args: *mut __va_list_tag,
) -> c_int {
    let trainer: &Trainer = unsafe { mem::transmute(user) };
    if !trainer.verbose {
        return 0;
    }
    unsafe {
        let mut buf = mem::MaybeUninit::<[c_char; 65535]>::uninit();
        let buf = {
            vsnprintf(buf.as_mut_ptr() as _, 65534, format, mem::transmute(args));
            buf.assume_init()
        };
        let message = CStr::from_ptr(buf.as_ptr()).to_str().unwrap();
        print!("{}", message);
    }
    0
}

impl Trainer {
    /// Construct a trainer
    pub fn new(verbose: bool) -> Self {
        unsafe {
            let data_ptr = libc::malloc(mem::size_of::<crfsuite_data_t>()) as *mut crfsuite_data_t;
            if !data_ptr.is_null() {
                crfsuite_data_init(data_ptr);
            }
            Self {
                data: data_ptr,
                trainer: ptr::null_mut(),
                verbose,
            }
        }
    }

    fn init(&mut self) -> Result<()> {
        unsafe {
            let interface = CString::new("dictionary").unwrap();
            if (*self.data).labels.is_null() {
                let ret = crfsuite_create_instance(
                    interface.as_ptr() as *const _,
                    &mut (*self.data).attrs as *mut *mut _ as *mut *mut _,
                );
                // ret is c bool
                if ret == 0 {
                    return Err(CrfError::CreateInstanceError(
                        "Failed to create a dictionary instance for attributes.".to_string(),
                    ));
                }
            }
            if (*self.data).labels.is_null() {
                let ret = crfsuite_create_instance(
                    interface.as_ptr() as *const _,
                    &mut (*self.data).labels as *mut *mut _ as *mut *mut _,
                );
                // ret is c bool
                if ret == 0 {
                    return Err(CrfError::CreateInstanceError(
                        "Failed to create a dictionary instance for labels.".to_string(),
                    ));
                }
            }
        }
        #[cfg(not(windows))]
        {
            self.set_message_callback();
        }
        Ok(())
    }

    /// Remove all instances in the data set
    pub fn clear(&mut self) -> Result<()> {
        if self.data.is_null() {
            return Ok(());
        }
        unsafe {
            if !(*self.data).attrs.is_null() {
                (*(*self.data).attrs)
                    .release
                    .map(|release| release((*self.data).attrs))
                    .unwrap();
                (*self.data).attrs = ptr::null_mut();
            }
            if !(*self.data).labels.is_null() {
                (*(*self.data).labels)
                    .release
                    .map(|release| release((*self.data).labels))
                    .unwrap();
                (*self.data).labels = ptr::null_mut();
            }
            crfsuite_data_finish(self.data);
            crfsuite_data_init(self.data);
        }
        Ok(())
    }

    /// Append an instance (item/label sequence) to the data set.
    ///
    /// ## Parameters
    ///
    /// `xseq`: a sequence of item features, The item sequence of the instance.
    ///
    /// `yseq`: a sequence of strings, The label sequence of the instance.
    ///
    /// `group`: The group number of the instance. Group numbers are used to select subset of data
    /// for heldout evaluation.
    pub fn append<T: AsRef<str>>(&mut self, xseq: &[Item], yseq: &[T], group: i32) -> Result<()> {
        unsafe {
            if (*self.data).attrs.is_null() || (*self.data).labels.is_null() {
                self.init()?;
            }
            let xseq_len = xseq.len();
            assert_eq!(xseq_len, yseq.len());
            let mut instance = mem::MaybeUninit::<crfsuite_instance_t>::uninit();
            let mut instance = {
                crfsuite_instance_init_n(instance.as_mut_ptr(), xseq_len as i32);
                instance.assume_init()
            };
            let crf_items = slice::from_raw_parts_mut(instance.items, instance.num_items as usize);
            let crf_labels =
                slice::from_raw_parts_mut(instance.labels, instance.num_items as usize);
            for t in 0..xseq_len {
                let items = &xseq[t];
                let crf_item = &mut crf_items[t];
                // Set the attributes in the item
                crfsuite_item_init_n(crf_item, items.len() as i32);
                let contents =
                    slice::from_raw_parts_mut(crf_item.contents, crf_item.num_contents as usize);
                for (content, item) in contents.iter_mut().zip(items) {
                    let name_cstr = CString::new(&item.name[..]).unwrap();
                    let aid = (*(*self.data).attrs)
                        .get
                        .map(|f| f((*self.data).attrs, name_cstr.as_ptr()))
                        .unwrap();
                    (*content).aid = aid;
                    (*content).value = item.value;
                }
                // Set the label of the item
                let y_value = yseq[t].as_ref();
                let y_cstr = CString::new(y_value).unwrap();
                let label = (*(*self.data).labels)
                    .get
                    .map(|f| f((*self.data).labels, y_cstr.as_ptr()))
                    .unwrap();
                crf_labels[t] = label;
            }
            instance.group = group;
            // Append the instance to the training set
            crfsuite_data_append(self.data, &instance);
            // Finish the instance
            crfsuite_instance_finish(&mut instance);
        }
        Ok(())
    }

    /// Initialize the training algorithm.
    pub fn select(&mut self, algorithm: Algorithm, typ: GraphicalModel) -> Result<()> {
        unsafe {
            // Release the trainer if it is already initialzed
            if !self.trainer.is_null() {
                (*self.trainer).release.map(|f| f(self.trainer)).unwrap();
                self.trainer = ptr::null_mut();
            }
            let mut tid = String::from("train/");
            tid.push_str(&typ.to_string());
            tid.push_str("/");
            tid.push_str(&algorithm.to_string());
            let tid_cstr = CString::new(tid).unwrap();
            let ret = crfsuite_create_instance(
                tid_cstr.as_ptr(),
                &mut self.trainer as *mut *mut _ as *mut *mut _,
            );
            // ret is c bool
            if ret == 0 {
                return Err(CrfError::CreateInstanceError(
                    "Failed to create a instance for trainer.".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Run the training algorithm.
    ///
    /// This function starts the training algorithm with the data set given
    /// by `append()` function.
    ///
    /// ## Parameters
    ///
    /// `model`: The filename to which the trained model is stored
    ///
    /// `holdout`: The group number of holdout evaluation.
    /// the instances with this group number will not be used
    /// for training, but for holdout evaluation.
    /// -1 meaning "use all instances for training".
    pub fn train(&mut self, model: &str, holdout: i32) -> Result<()> {
        if self.trainer.is_null() {
            return Err(CrfError::AlgorithmNotSelected);
        }
        unsafe {
            if (*self.data).attrs.is_null() || (*self.data).labels.is_null() {
                return Err(CrfError::EmptyData);
            }
            let model_cstr = CString::new(model).unwrap();
            let ret = (*self.trainer)
                .train
                .map(|f| f(self.trainer, self.data, model_cstr.as_ptr(), holdout))
                .unwrap();
            if ret != 0 {
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
        }
        Ok(())
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
                (*pms).name.map(|f| f(pms, i, &mut name)).unwrap();
                let c_str = CStr::from_ptr(name);
                ret.push(c_str.to_string_lossy().into_owned());
                (*pms).free.map(|f| f(pms, name)).unwrap();
            }
            (*pms).release.map(|f| f(pms)).unwrap();
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
            if (*pms)
                .set
                .map(|f| f(pms, name_cstr.as_ptr(), value_cstr.as_ptr()))
                .unwrap()
                != 0
            {
                (*pms).release.map(|f| f(pms)).unwrap();
                return Err(CrfError::ParamNotFound(name.to_string()));
            }
            (*pms).release.map(|f| f(pms)).unwrap();
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
            if (*pms)
                .get
                .map(|f| f(pms, name_cstr.as_ptr(), &mut value_ptr))
                .unwrap()
                != 0
            {
                (*pms).release.map(|f| f(pms)).unwrap();
                return Err(CrfError::ParamNotFound(name.to_string()));
            }
            value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();
            (*pms).free.map(|f| f(pms, value_ptr)).unwrap();
            (*pms).release.map(|f| f(pms)).unwrap();
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
            if (*pms)
                .help
                .map(|f| f(pms, name_cstr.as_ptr(), ptr::null_mut(), &mut value_ptr))
                .unwrap()
                != 0
            {
                (*pms).release.map(|f| f(pms)).unwrap();
                return Err(CrfError::ParamNotFound(name.to_string()));
            }
            value = CStr::from_ptr(value_ptr).to_string_lossy().into_owned();
            (*pms).free.map(|f| f(pms, value_ptr)).unwrap();
            (*pms).release.map(|f| f(pms)).unwrap();
        }
        Ok(value)
    }

    #[cfg(not(windows))]
    /// Set the callback function and user-defined data
    // XXX: make it a public API?
    fn set_message_callback(&mut self) {
        unsafe {
            (*self.trainer)
                .set_message_callback
                .map(|f| f(self.trainer, mem::transmute(self), Some(logging_callback)))
                .unwrap();
        }
    }
}

impl Drop for Trainer {
    fn drop(&mut self) {
        unsafe {
            if !self.data.is_null() {
                self.clear().unwrap();
                libc::free(self.data as *mut _);
                self.data = ptr::null_mut();
            }
            if !self.trainer.is_null() {
                (*self.trainer).release.map(|f| f(self.trainer)).unwrap();
                self.trainer = ptr::null_mut();
            }
        }
    }
}

/// The model
#[derive(Debug)]
pub struct Model(*mut crfsuite_model_t);

/// The tagger
/// provides the functionality for predicting label sequences for input sequences using a model.
#[derive(Debug)]
pub struct Tagger<'a> {
    model: &'a Model,
    tagger: *mut crfsuite_tagger_t,
}

impl Model {
    #[inline]
    fn new() -> Self {
        Model(ptr::null_mut())
    }

    /// Open a model file
    pub fn from_file(name: &str) -> Result<Self> {
        let mut file = File::open(name)
            .map_err(|err| CrfError::InvalidModel(format!("Failed to open model: {}", err)))?;
        Self::validate_model(&mut file)?;
        drop(file); // Close file

        let mut model = Model::new();
        model.open(name)?;
        Ok(model)
    }

    /// Create an instance of a model object from a model in memory
    pub fn from_memory(bytes: &[u8]) -> Result<Self> {
        let mut cdr = Cursor::new(bytes);
        Self::validate_model(&mut cdr)?;
        let mut instance = ptr::null_mut();
        unsafe {
            let ret = crfsuite_create_instance_from_memory(
                bytes.as_ptr() as *const c_void,
                bytes.len(),
                &mut instance,
            );
            if ret != 0 {
                return Err(CrfError::CreateInstanceError(
                    "Failed to create a model instance.".to_string(),
                ));
            }
        }
        let model: *mut crfsuite_sys::crfsuite_model_t = unsafe { mem::transmute(instance) };
        Ok(Model(model))
    }

    /// Validate model
    ///
    /// See https://github.com/chokkan/crfsuite/pull/24
    fn validate_model<R: Read + Seek>(reader: &mut R) -> Result<()> {
        // Check that file magic is correct
        let mut magic = [0; 4];
        reader.read_exact(&mut magic).map_err(|err| {
            CrfError::InvalidModel(format!("Failed to read model file magic: {}", err))
        })?;
        if &magic != b"lCRF" {
            return Err(CrfError::InvalidModel(
                "Invalid model file magic".to_string(),
            ));
        }
        // Make sure crfsuite won't read past allocated memory in case of incomplete header
        let pos = reader
            .seek(SeekFrom::End(0))
            .map_err(|err| CrfError::InvalidModel(format!("Invalid model: {}", err)))?;
        if pos <= 48 {
            // header size
            return Err(CrfError::InvalidModel(
                "Invalid model file header".to_string(),
            ));
        }
        Ok(())
    }

    /// Open a model file
    fn open(&mut self, name: &str) -> Result<()> {
        let name_cstr = CString::new(name).unwrap();
        unsafe {
            let ret = crfsuite_create_instance_from_file(
                name_cstr.as_ptr(),
                &mut self.0 as *mut *mut _ as *mut *mut _,
            );
            if ret != 0 {
                return Err(CrfError::CreateInstanceError(
                    "Failed to create a model instance.".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Close the model
    fn close(&mut self) {
        unsafe {
            if !self.0.is_null() {
                (*self.0).release.map(|f| f(self.0)).unwrap();
            }
        }
    }

    pub fn tagger(&self) -> Result<Tagger> {
        unsafe {
            let mut tagger = ptr::null_mut();
            let ret = (*self.0)
                .get_tagger
                .map(|f| f(self.0, &mut tagger))
                .unwrap();
            if ret != 0 {
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            Ok(Tagger {
                model: self,
                tagger,
            })
        }
    }

    #[cfg(unix)]
    /// Print the model in human-readable format
    ///
    /// ## Parameters
    ///
    /// `file`: Something convertable to file descriptor
    ///
    pub fn dump(&self, fd: RawFd) -> Result<()> {
        let c_mode = CString::new("w").unwrap();
        unsafe {
            let file = fdopen(fd, c_mode.as_ptr());
            if file.is_null() {
                panic!("fdopen failed");
            }
            let ret = (*self.0).dump.map(|f| f(self.0, file)).unwrap();
            if ret != 0 {
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            fclose(file);
        }
        Ok(())
    }

    #[cfg(windows)]
    /// Print the model in human-readable format
    ///
    /// ## Parameters
    ///
    /// `file`: Something convertable to file descriptor
    ///
    pub fn dump(&self, fd: RawHandle) -> Result<()> {
        unsafe {
            let fd = libc::open_osfhandle(fd as _, libc::O_RDWR);
            if fd == -1 {
                panic!("open_osfhandle failed");
            }
            let c_mode = CString::new("w").unwrap();
            let file = fdopen(fd, c_mode.as_ptr());
            if file.is_null() {
                panic!("fdopen failed");
            }
            let ret = (*self.0).dump.map(|f| f(self.0, file)).unwrap();
            if ret != 0 {
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            fclose(file);
        }
        Ok(())
    }

    #[cfg(unix)]
    /// Print the model in human-readable format to file
    ///
    /// ## Parameters
    ///
    /// `path`: Dump file path
    ///
    pub fn dump_file<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let file = File::create(path).expect("create file failed");
        self.dump(file.into_raw_fd())
    }

    #[cfg(windows)]
    /// Print the model in human-readable format to file
    ///
    /// ## Parameters
    ///
    /// `path`: Dump file path
    ///
    pub fn dump_file<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        let file = File::create(path).expect("create file failed");
        self.dump(file.into_raw_handle())
    }

    unsafe fn get_attrs(&self) -> Result<*mut crfsuite_dictionary_t> {
        let mut attrs: *mut crfsuite_dictionary_t = ptr::null_mut();
        let ret = (*self.0).get_attrs.map(|f| f(self.0, &mut attrs)).unwrap();
        if ret != 0 {
            return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
        }
        Ok(attrs)
    }

    unsafe fn get_labels(&self) -> Result<*mut crfsuite_dictionary_t> {
        let mut labels: *mut crfsuite_dictionary_t = ptr::null_mut();
        let ret = (*self.0)
            .get_labels
            .map(|f| f(self.0, &mut labels))
            .unwrap();
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
        unsafe {
            (*self.tagger).release.map(|f| f(self.tagger)).unwrap();
        }
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
                let mut label: *mut libc::c_char = ptr::null_mut();
                let ret = (*labels)
                    .to_string
                    .map(|f| f(labels, i, &mut label as *mut *mut _ as *mut *const _))
                    .unwrap();
                if ret != 0 {
                    (*labels).release.map(|f| f(labels)).unwrap();
                    return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
                }
                lseq.push(CStr::from_ptr(label).to_string_lossy().into_owned());
                (*labels).free.map(|f| f(labels, label)).unwrap();
            }
            (*labels).release.map(|f| f(labels)).unwrap();
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
            let attrs = self.model.get_attrs()?;
            let xseq_len = xseq.len();
            let mut instance = mem::MaybeUninit::<crfsuite_instance_t>::uninit();
            let mut instance = {
                crfsuite_instance_init_n(instance.as_mut_ptr(), xseq_len as i32);
                instance.assume_init()
            };
            let crf_items = slice::from_raw_parts_mut(instance.items, instance.num_items as usize);
            for t in 0..xseq_len {
                let items = &xseq[t];
                let crf_item = &mut crf_items[t];
                // Set the attributes in the item
                crfsuite_item_init(crf_item);
                for attr in items.iter() {
                    let name_cstr = CString::new(&attr.name[..]).unwrap();
                    let aid = (*attrs)
                        .to_id
                        .map(|f| f(attrs, name_cstr.as_ptr()))
                        .unwrap();
                    if aid >= 0 {
                        let mut cont = mem::MaybeUninit::<crfsuite_attribute_t>::uninit();
                        let cont = {
                            crfsuite_attribute_set(cont.as_mut_ptr(), aid, attr.value);
                            cont.assume_init()
                        };
                        crfsuite_item_append_attribute(crf_item, &cont);
                    }
                }
            }

            // Set the instance to the tagger
            let ret = (*self.tagger)
                .set
                .map(|f| f(self.tagger, &mut instance))
                .unwrap();
            if ret != 0 {
                (*attrs).release.map(|f| f(attrs)).unwrap();
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            crfsuite_instance_finish(&mut instance);
            (*attrs).release.map(|f| f(attrs)).unwrap();
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
            let mut score: floatval_t = 0.0;
            let mut paths: Vec<libc::c_int> = Vec::with_capacity(length as usize);
            let ret = (*self.tagger)
                .viterbi
                .map(|f| f(self.tagger, paths.as_mut_ptr(), &mut score))
                .unwrap();
            if ret != 0 {
                (*labels).release.map(|f| f(labels)).unwrap();
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            paths.set_len(length as usize);
            let mut yseq = Vec::with_capacity(length as usize);
            // Convert the Viterbi path to a label sequence
            for path in paths {
                let mut label: *mut libc::c_char = ptr::null_mut();
                let ret = (*labels)
                    .to_string
                    .map(|f| f(labels, path, &mut label as *mut *mut _ as *mut *const _))
                    .unwrap();
                if ret != 0 {
                    (*labels).release.map(|f| f(labels)).unwrap();
                    return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
                }
                yseq.push(CStr::from_ptr(label).to_string_lossy().into_owned());
                (*labels).free.map(|f| f(labels, label)).unwrap();
            }
            (*labels).release.map(|f| f(labels)).unwrap();
            Ok(yseq)
        }
    }

    /// Compute the probability of the label sequence.
    pub fn probability<T: AsRef<str>>(&self, yseq: &[T]) -> Result<f64> {
        let mut score: floatval_t = 0.0;
        unsafe {
            // Make sure that the current instance is not empty
            let length = (*self.tagger).length.map(|f| f(self.tagger)).unwrap() as usize;
            if length == 0 {
                return Ok(score);
            }
            // Make sure |y| == |x|
            if length != yseq.len() {
                return Err(CrfError::InvalidArgument(format!(
                    "The numbers of items and labels differ: |x| = {}, |y| = {}",
                    length,
                    yseq.len()
                )));
            }
            // Obtain the dictionary interface representing the labels in the model.
            let labels = self.model.get_labels()?;
            // Convert string labels into label IDs.
            let mut paths: Vec<libc::c_int> = Vec::with_capacity(length);
            for y in yseq.iter() {
                let y_cstr = CString::new(y.as_ref()).unwrap();
                let l = (*labels).to_id.map(|f| f(labels, y_cstr.as_ptr())).unwrap();
                if l < 0 {
                    (*labels).release.map(|f| f(labels)).unwrap();
                    return Err(CrfError::ValueError(format!(
                        "Failed to convert into label identifier: {}",
                        y.as_ref()
                    )));
                }
                paths.push(l);
            }
            // Compute the score of the path.
            let ret = (*self.tagger)
                .score
                .map(|f| f(self.tagger, paths.as_mut_ptr(), &mut score))
                .unwrap();
            if ret != 0 {
                (*labels).release.map(|f| f(labels)).unwrap();
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            // Compute the partition factor.
            let mut lognorm: floatval_t = 0.0;
            let ret = (*self.tagger)
                .lognorm
                .map(|f| f(self.tagger, &mut lognorm))
                .unwrap();
            (*labels).release.map(|f| f(labels)).unwrap();
            if ret != 0 {
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            Ok((score - lognorm).exp())
        }
    }

    /// Compute the marginal probability of the label.
    pub fn marginal(&self, label: &str, position: i32) -> Result<f64> {
        let mut prob: floatval_t = 0.0;
        unsafe {
            // Make sure that the current instance is not empty
            let length = (*self.tagger).length.map(|f| f(self.tagger)).unwrap() as usize;
            if length == 0 {
                return Ok(prob);
            }
            // Make sure that 0 <= position < |x|.
            if position < 0 || length <= position as usize {
                return Err(CrfError::InvalidArgument(format!(
                    "The position {} is out of range of {}",
                    position, length
                )));
            }
            // Obtain the dictionary interface representing the labels in the model.
            let labels = self.model.get_labels()?;
            // Convert string labels into label IDs.
            let y_cstr = CString::new(label).unwrap();
            let l = (*labels).to_id.map(|f| f(labels, y_cstr.as_ptr())).unwrap();
            if l < 0 {
                (*labels).release.map(|f| f(labels)).unwrap();
                return Err(CrfError::ValueError(format!(
                    "Failed to convert into label identifier: {}",
                    label
                )));
            }
            // Compute the score of the path.
            let ret = (*self.tagger)
                .marginal_point
                .map(|f| f(self.tagger, l, position, &mut prob))
                .unwrap();
            (*labels).release.map(|f| f(labels)).unwrap();
            if ret != 0 {
                return Err(CrfError::CrfSuiteError(CrfSuiteError::from(ret)));
            }
            Ok(prob)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Algorithm, Attribute, GraphicalModel, Result};

    #[test]
    fn test_str_to_algorithm_enum() {
        let algo: Algorithm = "lbfgs".parse().unwrap();
        assert_eq!(algo, Algorithm::LBFGS);

        let algo: Algorithm = "l2sgd".parse().unwrap();
        assert_eq!(algo, Algorithm::L2SGD);

        let algo: Algorithm = "ap".parse().unwrap();
        assert_eq!(algo, Algorithm::AP);
        let algo: Algorithm = "averaged-perceptron".parse().unwrap();
        assert_eq!(algo, Algorithm::AP);

        let algo: Algorithm = "pa".parse().unwrap();
        assert_eq!(algo, Algorithm::PA);
        let algo: Algorithm = "passive-aggressive".parse().unwrap();
        assert_eq!(algo, Algorithm::PA);

        let algo: Algorithm = "arow".parse().unwrap();
        assert_eq!(algo, Algorithm::AROW);

        let algo: Result<Algorithm> = "foo".parse();
        assert!(algo.is_err());
    }

    #[test]
    fn test_algorithm_enum_to_str() {
        assert_eq!("lbfgs", &Algorithm::LBFGS.to_string());
        assert_eq!("l2sgd", &Algorithm::L2SGD.to_string());
        assert_eq!("averaged-perceptron", &Algorithm::AP.to_string());
        assert_eq!("passive-aggressive", &Algorithm::PA.to_string());
        assert_eq!("arow", &Algorithm::AROW.to_string());
    }

    #[test]
    fn test_str_to_graphical_model_enum() {
        let model: GraphicalModel = "1d".parse().unwrap();
        assert_eq!(model, GraphicalModel::CRF1D);
        let model: GraphicalModel = "crf1d".parse().unwrap();
        assert_eq!(model, GraphicalModel::CRF1D);

        let model: Result<GraphicalModel> = "foo".parse();
        assert!(model.is_err());
    }

    #[test]
    fn test_attribute() {
        Attribute::new("foo", 1.0);
        Attribute::from(("foo", 1.0));
        assert_eq!(Attribute::from("foo"), Attribute::from(("foo", 1.0)));
    }
}
