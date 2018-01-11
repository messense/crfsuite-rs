# -*- coding: utf-8 -*-
from __future__ import absolute_import
import weakref

from ._compat import text_type, NUL
from ._native import lib, ffi
from .exceptions import exceptions_by_code, CrfSuiteError

attached_refs = weakref.WeakKeyDictionary()


def rustcall(func, *args):
    """Calls rust method and does some error handling."""
    lib.pycrfsuite_err_clear()
    rv = func(*args)
    err = lib.pycrfsuite_err_get_last_code()
    if not err:
        return rv
    msg = lib.pycrfsuite_err_get_last_message()
    cls = exceptions_by_code.get(err, CrfSuiteError)
    exc = cls(decode_str(msg))
    raise exc


def decode_str(s, free=False):
    """Decodes a FfiStr"""
    try:
        if s.len == 0:
            return u''
        return ffi.unpack(s.data, s.len).decode('utf-8', 'replace')
    finally:
        if free:
            lib.pycrfsuite_str_free(ffi.addressof(s))


def encode_str(s):
    """Encodes a FfiStr"""
    rv = ffi.new('FfiStr *')
    if isinstance(s, text_type):
        s = s.encode('utf-8')
    rv.data = ffi.from_buffer(s)
    rv.len = len(s)
    # we have to hold a weak reference here to ensure our string does not
    # get collected before the string is used.
    attached_refs[rv] = s
    return rv
