# -*- coding: utf-8 -*-
from __future__ import absolute_import, print_function
import os
import sys

from ._compat import text_type, string_types
from ._native import lib, ffi
from .utils import rustcall, decode_str

# Make sure we init the lib and turn on rust backtraces
os.environ['RUST_BACKTRACE'] = '1'
ffi.init_once(lib.pycrfsuite_init, 'init')

class Model(object):
    def __init__(self, model_path=None, model_bytes=None):
        assert model_path or model_bytes, 'model_path or model_bytes requried'
        assert not (model_path and model_bytes), 'model_path and model_bytes should not be used together'

        if model_path:
            if isinstance(model_path, text_type):
                model_path = model_path.encode('utf-8')
            self.model = ffi.gc(rustcall(lib.pycrfsuite_model_open, model_path), lib.pycrfsuite_model_destroy)

        if model_bytes:
            if isinstance(model_bytes, text_type):
                model_bytes = model_bytes.encode('utf-8')
            self.model = ffi.gc(
                rustcall(lib.pycrfsuite_model_from_bytes, model_bytes, len(model_bytes)),
                lib.pycrfsuite_model_destroy
            )

    def tag(self, xseq):
        if not xseq:
            return []
        tagger = Tagger(self.model)
        return tagger.tag(xseq)

    def dump(self, filename=None):
        if filename is None:
            rustcall(lib.pycrfsuite_model_dump, self.model, os.dup(sys.stdout.fileno()))
        else:
            fd = os.open(filename, os.O_CREAT | os.O_WRONLY)
            try:
                rustcall(lib.pycrfsuite_model_dump, self.model, fd)
            finally:
                try:
                    os.close(fd)
                except OSError:
                    pass  # Already closed


def _to_attr(x):
    if isinstance(x, tuple):
        name = x[0]
        value = x[1]
    elif isinstance(x, string_types):
        name = x
        value = 1.0

    if isinstance(name, text_type):
        name = name.encode('utf-8')
    return (name, value)


class Tagger(object):
    def __init__(self, model):
        self.tagger = ffi.gc(rustcall(lib.pycrfsuite_tagger_create, model), lib.pycrfsuite_tagger_destroy)

    def tag(self, xseq):
        attrs_list = ffi.new('AttributeList []', len(xseq))
        keepalive = []
        for i, items in enumerate(xseq):
            attrs = attrs_list[i]
            attrs.len = len(items)
            attr_ptr = ffi.new('Attribute []', len(items))
            keepalive.append(attr_ptr)
            for j, item in enumerate(items):
                attr = attr_ptr[j]
                name, value = _to_attr(item)
                name = ffi.from_buffer(name)
                keepalive.append(name)
                attr.name = name
                attr.value = ffi.cast('double', value)
            attrs.data = attr_ptr

        tags = rustcall(lib.pycrfsuite_tagger_tag, self.tagger, attrs_list, len(xseq))
        ffi_strs = ffi.unpack(tags.data, tags.len)
        labels = [decode_str(s) for s in ffi_strs]
        lib.pycrfsuite_tags_destroy(tags)
        return labels


def _intbool(txt):
    return bool(int(txt))


class Trainer(object):

    _PARAMETER_TYPES = {
        'feature.minfreq': float,
        'feature.possible_states': _intbool,
        'feature.possible_transitions': _intbool,
        'c1': float,
        'c2': float,
        'max_iterations': int,
        'num_memories': int,
        'epsilon': float,
        'period': int,  # XXX: is it called 'stop' in docs?
        'delta': float,
        'linesearch': str,
        'max_linesearch': int,
        'calibration.eta': float,
        'calibration.rate': float,
        'calibration.samples': float,
        'calibration.candidates': int,
        'calibration.max_trials': int,
        'type': int,
        'c': float,
        'error_sensitive': _intbool,
        'averaging': _intbool,
        'variance': float,
        'gamma': float,
    }

    def __init__(self, algorithm='lbfgs', verbose=False):
        self.trainer = ffi.gc(rustcall(lib.pycrfsuite_trainer_create, bool(verbose)), lib.pycrfsuite_trainer_destroy)
        self.select(algorithm)

    def select(self, algorithm):
        if isinstance(algorithm, text_type):
            algorithm = algorithm.encode('utf-8')
        rustcall(lib.pycrfsuite_trainer_select, self.trainer, algorithm)

    def train(self, model_path, holdout=-1):
        if isinstance(model_path, text_type):
            model_path = model_path.encode('utf-8')
        rustcall(lib.pycrfsuite_trainer_train, self.trainer, model_path, holdout)

    def clear(self):
        rustcall(lib.pycrfsuite_trainer_clear, self.trainer)

    def append(self, xseq, yseq, group=0):
        attrs_list = ffi.new('AttributeList []', len(xseq))
        keepalive = []
        for i, items in enumerate(xseq):
            attrs = attrs_list[i]
            attrs.len = len(items)
            attr_ptr = ffi.new('Attribute []', len(items))
            keepalive.append(attr_ptr)
            for j, item in enumerate(items):
                attr = attr_ptr[j]
                name, value = _to_attr(item)
                name = ffi.from_buffer(name)
                keepalive.append(name)
                attr.name = name
                attr.value = ffi.cast('double', value)
            attrs.data = attr_ptr
        tag_list = ffi.new('char* []', len(yseq))
        for i, tag in enumerate(yseq):
            if isinstance(tag, text_type):
                tag = tag.encode('utf-8')
            tag = ffi.from_buffer(tag)
            keepalive.append(tag)
            tag_list[i] = tag

        rustcall(
            lib.pycrfsuite_trainer_append,
            self.trainer,
            attrs_list,
            len(xseq),
            tag_list,
            len(yseq),
            group
        )

    def get(self, name):
        if isinstance(name, text_type):
            c_name = name.encode('utf-8')
        else:
            c_name = name
        value = rustcall(lib.pycrfsuite_trainer_get, self.trainer, c_name)
        return self._cast_parameter(name, decode_str(value, free=True))

    def get_params(self):
        return {name: self.get(name) for name in self.params()}

    def set(self, name, value):
        if isinstance(name, text_type):
            name = name.encode('utf-8')
        if isinstance(value, bool):
            value = int(value)
        value = str(value).encode('utf-8')
        rustcall(lib.pycrfsuite_trainer_set, self.trainer, name, value)

    def set_params(self, params):
        for name, value in params.items():
            self.set(name, value)

    def help(self, name):
        if isinstance(name, text_type):
            name = name.encode('utf-8')
        value = rustcall(lib.pycrfsuite_trainer_help, self.trainer, name)
        return decode_str(value, free=True)

    def params(self):
        c_params = rustcall(lib.pycrfsuite_trainer_params, self.trainer)
        ffi_strs = ffi.unpack(c_params.data, c_params.len)
        params = [decode_str(s) for s in ffi_strs]
        lib.pycrfsuite_params_destroy(c_params)
        return params

    def _cast_parameter(self, name, value):
        if name in self._PARAMETER_TYPES:
            return self._PARAMETER_TYPES[name](value)
        return value
