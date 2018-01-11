# -*- coding: utf-8 -*-
from __future__ import absolute_import, print_function
import os

from ._compat import text_type
from ._native import lib, ffi
from .utils import rustcall, decode_str

# Make sure we init the lib and turn on rust backtraces
os.environ['RUST_BACKTRACE'] = '1'
ffi.init_once(lib.pycrfsuite_init, 'init')

class Model(object):
    def __init__(self, model_path):
        if isinstance(model_path, text_type):
            model_path = model_path.encode('utf-8')
        self.model = rustcall(lib.pycrfsuite_model_open, model_path)

    def __del__(self):
        if getattr(self, 'model', None):
            lib.pycrfsuite_model_destroy(self.model)
            self.model = None

    def tag(self, xseq):
        if not xseq:
            return []
        tagger = Tagger(self.model)
        return tagger.tag(xseq)


class Tagger(object):
    def __init__(self, model):
        self.tagger = rustcall(lib.pycrfsuite_tagger_create, model)

    def __del__(self):
        if getattr(self, 'tagger', None):
            lib.pycrfsuite_tagger_destroy(self.tagger)

    def tag(self, xseq):
        attrs_list = ffi.new('AttributeList []', len(xseq));
        keepalive = []
        for i, items in enumerate(xseq):
            attrs = attrs_list[i]
            attrs.len = len(items)
            attr_ptr = ffi.new('Attribute []', len(items))
            keepalive.append(attr_ptr)
            for j, item in enumerate(items):
                attr = attr_ptr[j]
                if isinstance(item[0], text_type):
                    s = item[0].encode('utf-8')
                else:
                    s = item[0]
                name = ffi.from_buffer(s)
                keepalive.append(name)
                attr.name = name
                attr.value = ffi.cast('double', item[1])
            attrs.data = attr_ptr

        tags = rustcall(lib.pycrfsuite_tagger_tag, self.tagger, attrs_list, len(xseq))
        ffi_strs = ffi.unpack(tags.data, tags.len)
        labels = [decode_str(s) for s in ffi_strs]
        lib.pycrfsuite_tags_destroy(tags)
        return labels
