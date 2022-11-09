FROM quay.io/pypa/manylinux2014_x86_64

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    PYBIN=/opt/python/cp37-cp37m/bin
    
RUN git config --global --add safe.directory /app

RUN curl https://sh.rustup.rs -sSf | sh -s -- --no-modify-path -y; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN for PYBIN in /opt/python/{cp37-cp37m,cp38-cp38,cp39-cp39,cp310-cp310,cp311-cp311}/bin; do \
  echo "--> $PYBIN"; \
  $PYBIN/pip install \
    numpy \
    cffi \
    setuptools \
    pytest \
    twine \
    black==22.3.0; \
  done

