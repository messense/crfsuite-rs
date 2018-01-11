# -*- coding: utf-8 -*-
from __future__ import absolute_import

from ._compat import implements_to_string
from ._native import lib


exceptions_by_code = {}


@implements_to_string
class CrfSuiteError(Exception):
    code = None

    def __init__(self, msg):
        Exception.__init__(self)
        self.message = msg
        self.rust_info = None

    def __str__(self):
        rv = self.message
        if self.rust_info is not None:
            return u'%s\n\n%s' % (rv, self.rust_info)
        return rv


def _make_exceptions():
    for attr in dir(lib):
        if not attr.startswith('CRF_ERROR_CODE_'):
            continue

        class Exc(CrfSuiteError):
            pass

        Exc.__name__ = attr[15:].title().replace('_', '')
        Exc.code = getattr(lib, attr)
        globals()[Exc.__name__] = Exc
        exceptions_by_code[Exc.code] = Exc


_make_exceptions()
