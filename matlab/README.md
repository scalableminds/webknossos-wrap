# webKNOSSOS Wrapper for MATLAB
This directory contains the code for creating, reading, writing, and
compressing webKNOSSOS wrapper datasets from within MATLAB. This library
is used in production at the Max Plack Institute for Brain Research
since mid-2017.

## Example
```matlab
% Create a new WKW dataset in `/home/amotta/data/wkw' with a single
% channel of `uint8` voxel data. The dataset will be organized in WKW
% files with (32 * 32)³ voxel cubes.
wkwInit('new', '/home/amotta/data/wkw', 32, 32, 'uint8', 1);

% Write a (128)³ voxel volume of random data to position (256, 256, 256)
% in the WKW dataset.
data = randi(intmax('uint8'), [128, 128, 128], 'uint8');
wkwSaveRoi('/home/amotta/data/wkw', [256, 256, 256], data);

% Read a (256)³ voxel volume starting at position [1, 1, 1].
box = [1, 1 + 256; 1, 1 + 256; 1, 1 + 256];
data = wkwLoadRoi('/home/amotta/data/wkw', box);
```

Each function is documented in-depth in its companion .m file.

### Windows
If you're using Windows, make sure that the latest version of the
[Microsoft Visual C++ Redistributable for Visual Studio 2015](https://support.microsoft.com/en-us/help/2977003/the-latest-supported-visual-c-downloads)
is installed on your system.

### macOS
Please note that this library was not yet tested on macOS. Only minor
modifications to `wkwBuild.m` and `lz4.rs` should be necessary to build
and run it. Contributions are welcome!

### Linux
If you're using Linux, make sure that `liblz4` is installed on your
system. On most Linux distributions this library is part of the official
package repository.

## How to build the library
To build the MEX files from the Rust code, just run the following
function from within MATLAB:
```matlab
>> wkwBuild();
```

This requires the [Rust compiler and build tools](https://www.rust-lang.org/en-US/install.html)
to be installed on your machine. If you're using Linux, then you will
furthermore need the development version of `liblz4` (`lz4` on Arch
Linux, `liblz4-dev` on Debian and Ubuntu).

## Contact
Contributions and bug reports are welcome!

- Alessandro Motta (alessandro.motta@brain.mpg.de)

