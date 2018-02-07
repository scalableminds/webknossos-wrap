# webKNOSSOS Wrapper for Python
This directory contains the code for creating, reading, writing, and
compressing webKNOSSOS wrapper datasets from Python. **WARNING:** This
library is experimental. It is **not ready for production**.

## Example
```python
import wkw
import numpy as np

# Create a WKW dataset
dataset = wkw.Dataset.create('./wkw', wkw.Header(np.uint8))

# Open a WKW dataset
dataset = wkw.Dataset.open('./wkw')

# Read a (128)³ voxel cube starting from position (0, 0, 0)
data = dataset.read([0, 0, 0], [128, 128, 128])
```

## How to build the library
To build and install this Python package, just run
```bash
$ python setup.py install
```

This requires the [Rust compiler and build tools](https://www.rust-lang.org/en-US/install.html)
to be installed on your machine. If you're using Linux, then you will
furthermore need the development version of `liblz4` (`lz4` on Arch
Linux, `liblz4-dev` on Debian and Ubuntu).

## Contact
Contributions and bug reports are welcome!

- Alessandro Motta (alessandro.motta@brain.mpg.de)

