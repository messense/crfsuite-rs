use std::boxed::Box;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::{fmt, mem, ptr, slice};

#[macro_use]
mod utils;

use utils::{set_panic_hook, LAST_ERROR};

#[derive(Debug)]
pub enum ErrorKind {
    Panic(String),
    CrfError(crfsuite::CrfError),
}

pub type Result<T> = ::std::result::Result<T, ErrorKind>;

impl ::std::error::Error for ErrorKind {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Panic(ref err) => err.fmt(f),
            ErrorKind::CrfError(ref err) => err.fmt(f),
        }
    }
}

impl From<crfsuite::CrfError> for ErrorKind {
    fn from(err: crfsuite::CrfError) -> ErrorKind {
        ErrorKind::CrfError(err)
    }
}

/// Represents a string.
#[repr(C)]
pub struct FfiStr {
    pub data: *mut c_char,
    pub len: usize,
    pub owned: bool,
}

impl Default for FfiStr {
    fn default() -> Self {
        Self {
            data: ptr::null_mut(),
            len: 0,
            owned: false,
        }
    }
}

impl FfiStr {
    pub fn from_string(mut s: String) -> Self {
        s.shrink_to_fit();
        let rv = Self {
            data: s.as_ptr() as *mut c_char,
            len: s.len(),
            owned: true,
        };
        mem::forget(s);
        rv
    }

    pub unsafe fn free(&mut self) {
        if self.owned && !self.data.is_null() {
            String::from_raw_parts(self.data as *mut _, self.len, self.len);
            self.data = ptr::null_mut();
            self.len = 0;
            self.owned = false;
        }
    }
}

impl Drop for FfiStr {
    fn drop(&mut self) {
        unsafe {
            self.free();
        }
    }
}

ffi_fn! {
    /// Creates a ffi str from a c string.
    ///
    /// This sets the string to owned.  In case it's not owned you either have
    /// to make sure you are not freeing the memory or you need to set the
    /// owned flag to false.
    unsafe fn pycrfsuite_str_from_cstr(s: *const c_char) -> Result<FfiStr> {
        let s = CStr::from_ptr(s).to_str().unwrap();
        Ok(FfiStr {
            data: s.as_ptr() as *mut _,
            len: s.len(),
            owned: true,
        })
    }
}

/// Frees a ffi str.
///
/// If the string is marked as not owned then this function does not
/// do anything.
#[no_mangle]
pub unsafe extern "C" fn pycrfsuite_str_free(s: *mut FfiStr) {
    if !s.is_null() {
        (*s).free()
    }
}

#[repr(u32)]
pub enum CrfErrorCode {
    NoError = 0,
    Panic = 1,
    CrfError = 2,
}

impl CrfErrorCode {
    pub fn from_kind(kind: &ErrorKind) -> Self {
        match *kind {
            ErrorKind::Panic(_) => CrfErrorCode::Panic,
            ErrorKind::CrfError(_) => CrfErrorCode::CrfError,
        }
    }
}

/// Initializes the library
#[no_mangle]
pub unsafe extern "C" fn pycrfsuite_init() {
    set_panic_hook();
}

#[no_mangle]
pub unsafe extern "C" fn pycrfsuite_err_get_last_code() -> CrfErrorCode {
    LAST_ERROR.with(|e| {
        if let Some(ref err) = *e.borrow() {
            CrfErrorCode::from_kind(err)
        } else {
            CrfErrorCode::NoError
        }
    })
}

/// Returns the last error message.
///
/// If there is no error an empty string is returned.  This allocates new memory
/// that needs to be freed with `pycrfsuite_str_free`.
#[no_mangle]
pub unsafe extern "C" fn pycrfsuite_err_get_last_message() -> FfiStr {
    use std::error::Error;
    use std::fmt::Write;

    LAST_ERROR.with(|e| {
        if let Some(ref err) = *e.borrow() {
            let mut msg = err.to_string();
            let mut cause = err.source();
            while let Some(the_cause) = cause {
                write!(&mut msg, "\n  caused by: {}", the_cause).ok();
                cause = the_cause.source();
            }
            FfiStr::from_string(msg)
        } else {
            Default::default()
        }
    })
}

/// Clears the last error.
#[no_mangle]
pub unsafe extern "C" fn pycrfsuite_err_clear() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

pub struct Model;
pub struct Tagger;
pub struct Trainer;

ffi_fn! {
    unsafe fn pycrfsuite_model_open(s: *const c_char) -> Result<*mut Model> {
        let path_cstr = CStr::from_ptr(s);
        let model = crfsuite::Model::from_file(path_cstr.to_str().unwrap())?;
        Ok(Box::into_raw(Box::new(model)) as *mut Model)
    }
}

#[cfg(unix)]
ffi_fn! {
    unsafe fn pycrfsuite_model_dump(m: *mut Model, fd: c_int) -> Result<()> {
        let model = m as *mut crfsuite::Model;
        Ok((*model).dump(fd)?)
    }
}

#[cfg(windows)]
ffi_fn! {
    unsafe fn pycrfsuite_model_dump(m: *mut Model, fd: c_int) -> Result<()> {
        let model = m as *mut crfsuite::Model;
        let handle = libc::get_osfhandle(fd);
        Ok((*model).dump(handle as _)?)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_model_destroy(m: *mut Model) {
        if !m.is_null() {
            let model = m as *mut crfsuite::Model;
            Box::from_raw(model);
        }
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_model_from_bytes(bytes: *const u8, len: usize) -> Result<*mut Model> {
        let bytes = slice::from_raw_parts(bytes, len);
        let model = crfsuite::Model::from_memory(bytes)?;
        Ok(Box::into_raw(Box::new(model)) as *mut Model)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_tagger_create(m: *mut Model) -> Result<*mut Tagger> {
        let model = m as *mut crfsuite::Model;
        let tagger =(*model).tagger()?;
        Ok(Box::into_raw(Box::new(tagger)) as *mut Tagger)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_tagger_destroy(t: *mut Tagger) {
        if !t.is_null() {
            let tagger = t as *mut crfsuite::Tagger;
            Box::from_raw(tagger);
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Attribute {
    pub name: *const c_char,
    pub value: f64,
}

#[repr(C)]
#[derive(Debug)]
pub struct AttributeList {
    data: *mut Attribute,
    len: usize,
}

#[repr(C)]
#[derive(Debug)]
pub struct Tags {
    pub data: *mut FfiStr,
    pub len: usize,
}

ffi_fn! {
    unsafe fn pycrfsuite_tags_destroy(tags: *mut Tags) {
        if !tags.is_null() {
            Vec::from_raw_parts((*tags).data, (*tags).len, (*tags).len);
            Box::from_raw(tags);
        }
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_tagger_tag(t: *mut Tagger, xseq: *const AttributeList, xseq_len: usize) -> Result<*mut Tags> {
        let items = slice::from_raw_parts(xseq, xseq_len);
        let mut x = Vec::with_capacity(items.len());
        for item in items {
            let attr_slice = slice::from_raw_parts(item.data, item.len);
            let attrs: Vec<crfsuite::Attribute> = attr_slice.iter()
                .map(|attr| crfsuite::Attribute::new(CStr::from_ptr(attr.name).to_string_lossy().to_owned(), attr.value))
                .collect();
            x.push(attrs);
        }
        let tagger = t as *mut crfsuite::Tagger;
        let labels = (*tagger).tag(&x)?;
        let mut tags: Vec<FfiStr> = labels.into_iter()
            .map(|l| FfiStr::from_string(l))
            .collect();
        tags.shrink_to_fit();
        let tag_count = tags.len();
        let buffer = tags.as_mut_ptr();
        mem::forget(tags);
        let c_tags = Tags { data: buffer, len: tag_count};
        Ok(Box::into_raw(Box::new(c_tags)))
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_create(verbose: bool) -> Result<*mut Trainer> {
        let trainer = crfsuite::Trainer::new(verbose);
        Ok(Box::into_raw(Box::new(trainer)) as *mut Trainer)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_destroy(trainer: *mut Trainer) {
        if !trainer.is_null() {
            let trainer = trainer as *mut crfsuite::Trainer;
            Box::from_raw(trainer);
        }
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_select(trainer: *mut Trainer, algo: *const c_char) -> Result<()> {
        let algorithm = CStr::from_ptr(algo)
            .to_str()
            .unwrap()
            .parse::<crfsuite::Algorithm>()?;
        let trainer = trainer as *mut crfsuite::Trainer;
        Ok((*trainer).select(algorithm, crfsuite::GraphicalModel::CRF1D)?)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_clear(trainer: *mut Trainer) -> Result<()> {
        let trainer = trainer as *mut crfsuite::Trainer;
        Ok((*trainer).clear()?)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_train(trainer: *mut Trainer, model: *const c_char, holdout: c_int) -> Result<()> {
        let trainer = trainer as *mut crfsuite::Trainer;
        let model_str = CStr::from_ptr(model).to_str().unwrap();
        Ok((*trainer).train(model_str, holdout as i32)?)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_append(trainer: *mut Trainer, xseq: *const AttributeList, xseq_len: usize, yseq: *const *const c_char, yseq_len: usize, group: c_int) -> Result<()> {
        let trainer = trainer as *mut crfsuite::Trainer;
        let items = slice::from_raw_parts(xseq, xseq_len);
        let mut x = Vec::with_capacity(items.len());
        for item in items {
            let attr_slice = slice::from_raw_parts(item.data, item.len);
            let attrs: Vec<crfsuite::Attribute> = attr_slice.iter()
                .map(|attr| crfsuite::Attribute::new(CStr::from_ptr(attr.name).to_string_lossy().to_owned(), attr.value))
                .collect();
            x.push(attrs);
        }
        let items = slice::from_raw_parts(yseq, yseq_len);
        let mut y = Vec::with_capacity(items.len());
        for item in items {
            let tag = CStr::from_ptr(*item).to_str().unwrap();
            y.push(tag);
        }
        Ok((*trainer).append(&x, &y, group)?)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_set(trainer: *mut Trainer, name: *const c_char, value: *const c_char) -> Result<()> {
        let trainer = trainer as *mut crfsuite::Trainer;
        let name_str = CStr::from_ptr(name).to_str().unwrap();
        let value_str = CStr::from_ptr(value).to_str().unwrap();
        Ok((*trainer).set(name_str, value_str)?)
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_get(trainer: *mut Trainer, name: *const c_char) -> Result<FfiStr> {
        let trainer = trainer as *mut crfsuite::Trainer;
        let name_str = CStr::from_ptr(name).to_str().unwrap();
        let value = (*trainer).get(name_str)?;
        Ok(FfiStr::from_string(value))
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_help(trainer: *mut Trainer, name: *const c_char) -> Result<FfiStr> {
        let trainer = trainer as *mut crfsuite::Trainer;
        let name_str = CStr::from_ptr(name).to_str().unwrap();
        let value = (*trainer).help(name_str)?;
        Ok(FfiStr::from_string(value))
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Params {
    pub data: *mut FfiStr,
    pub len: usize,
}

ffi_fn! {
    unsafe fn pycrfsuite_trainer_params(trainer: *mut Trainer) -> Result<*mut Params> {
        let trainer = trainer as *mut crfsuite::Trainer;
        let params = (*trainer).params();
        let mut params: Vec<FfiStr> = params.into_iter()
            .map(|l| FfiStr::from_string(l))
            .collect();
        params.shrink_to_fit();
        let param_count = params.len();
        let buffer = params.as_mut_ptr();
        mem::forget(params);
        let c_params = Params { data: buffer, len: param_count};
        Ok(Box::into_raw(Box::new(c_params)))
    }
}

ffi_fn! {
    unsafe fn pycrfsuite_params_destroy(params: *mut Params) {
        if !params.is_null() {
            Vec::from_raw_parts((*params).data, (*params).len, (*params).len);
            Box::from_raw(params);
        }
    }
}
