# webKNOSSOS Wrapper for MATLAB
This directory contains the code needed to create, read, write, and
compress webKNOSSOS wrapper datasets from within MATLAB.

## Example
```
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

## Linux
If you're using Linux, make sure that `liblz4` is installed on your
system. On most Linux distributions this library is part of the official
package repository.

## How to build the library
To build the MEX files from the Rust code, just run the following
function from within MATLAB:
```
>> wkwBuild();
```

This requires the [Rust compiler and build tools](https://www.rust-lang.org/en-US/install.html)
to be installed on your machine. If you're using Linux, then you will
furthermore need the development version of `liblz4` (`lz4` on Arch
Linux, `liblz4-dev` on Debian and Ubuntu).

## Contact
Contributions and bug reports are welcome!

- Alessandro Motta <alessandro.motta@brain.mpg.de>

