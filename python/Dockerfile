FROM quay.io/pypa/manylinux1_x86_64

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    PYBIN=/opt/python/cp36-cp36m/bin

RUN yum install lz4 lz4-devel -y; \
    curl https://sh.rustup.rs -sSf | sh -s -- --no-modify-path -y; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN $PYBIN/pip install \
  numpy \
  cffi \
  setuptools \
  pytest \
  twine \
  black
