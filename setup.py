# -*- coding: utf-8 -*-
from setuptools import setup, find_packages


def build_native(spec):
    # build an example rust library
    build = spec.add_external_build(
        cmd=['cargo', 'build', '--release'],
        path='./cabi'
    )

    spec.add_cffi_module(
        module_path='crfsuite._native',
        dylib=lambda: build.find_dylib('pycrfsuite', in_path='target/release'),
        header_filename=lambda: build.find_header('pycrfsuite.h', in_path='include'),
        rtld_flags=['NOW', 'NODELETE']
    )


setup(
    name='crfsuite',
    version='0.2.5',
    url='https://github.com/bosondata/crfsuite-rs',
    description='Python binding for crfsuite',
    packages=find_packages(),
    zip_safe=False,
    platforms='any',
    setup_requires=['milksnake'],
    install_requires=['milksnake'],
    milksnake_tasks=[
        build_native
    ]
)
