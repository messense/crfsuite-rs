#!/bin/bash
set -e -x

ln -s `which cmake28` /usr/bin/cmake

mkdir ~/rust-installer
curl -sL https://static.rust-lang.org/rustup.sh -o ~/rust-installer/rustup.sh
sh ~/rust-installer/rustup.sh --prefix=~/rust --spec=nightly -y --disable-sudo
export PATH="$HOME/rust/bin:$PATH"

# Compile wheels
for PYBIN in /opt/python/cp36*/bin; do
    export PYTHON_LIB=$(${PYBIN}/python -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
    export LIBRARY_PATH="$LIBRARY_PATH:$PYTHON_LIB"
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB:$HOME/rust/lib"
    "${PYBIN}/pip" install -U  setuptools setuptools-rust wheel
    "${PYBIN}/pip" wheel /io/python/ -w /io/python/dist/
done

# Bundle external shared libraries into the wheels
for whl in /io/python/dist/crfsuite*.whl; do
    auditwheel repair "$whl" -w /io/python/dist/
done
