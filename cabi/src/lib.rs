extern crate backtrace;
extern crate crfsuite;

use std::{fmt, mem, ptr, slice, str};
use std::boxed::Box;
use std::ffi::CStr;
use std::os::raw::c_char;

#[macro_use]
mod utils;

use utils::{set_panic_hook, LAST_ERROR};

#[derive(Debug)]
pub enum ErrorKind {
    Panic(String),
    CrfError(crfsuite::CrfError),
}

pub type Result<T> = ::std::result::Result<T, ErrorKind>;

impl ::std::error::Error for ErrorKind {
    fn description(&self) -> &str {
        match *self {
            ErrorKind::Panic(_) => "panic",
            ErrorKind::CrfError(ref err) => err.description(),
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Panic(ref err) => err.fmt(f),
            ErrorKind::CrfError(ref err) => err.fmt(f),
        }
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
        unsafe { self.free(); }
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
    use std::fmt::Write;
    use std::error::Error;

    LAST_ERROR.with(|e| {
        if let Some(ref err) = *e.borrow() {
            let mut msg = err.to_string();
            let mut cause = err.cause();
            while let Some(the_cause) = cause {
                write!(&mut msg, "\n  caused by: {}", the_cause).ok();
                cause = the_cause.cause();
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

ffi_fn! {
    unsafe fn pycrfsuite_model_open(s: *const c_char) -> Result<*mut Model> {
        let path_cstr = CStr::from_ptr(s);
        let model = crfsuite::Model::from_file(path_cstr.to_str().unwrap())
            .map_err(ErrorKind::CrfError)?;
        Ok(Box::into_raw(Box::new(model)) as *mut Model)
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
    unsafe fn pycrfsuite_tagger_create(m: *mut Model) -> Result<*mut Tagger> {
        let model = m as *mut crfsuite::Model;
        let tagger =(*model).tagger().map_err(ErrorKind::CrfError)?;
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
        let labels = (*tagger).tag(&x).map_err(ErrorKind::CrfError)?;
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
