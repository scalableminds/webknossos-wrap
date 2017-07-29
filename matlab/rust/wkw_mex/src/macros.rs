/*
MATLAB expects there to be (at least?) two symbols in a .mexa64 file. (Note that I'm only
considering linux here.) These are:

mexFunction, containing the user-defined code. The expected signature of this function is
publicly documented on MATHWORK's website. See
http:// mathworks.com/help/matlab/apiref/mexfunction.html

mexfilerequiredapiversion is the second and undocumented function. When invoking the mex command
with the -v (verbose) flag, one sees that it also builds (and later links) a second C++ file.
This file, namely MATLABROOT/extern/version/cpp_mxapi_version.cpp, has the following content:

(Content omited here to avoid copyright infringement.)

It sets the two variables pointed at by built_by_rel and target_api_version to 0x2016b (nice hex
code for the MATLAB version used during compilation) and 0x07300000, respectively. The first
constant is contained in the C++ file, while the second value is defined in the
MATLABROOT/extern/include/matrix.h header file that gets included by mex.h via
cpp_mxapi_version.cpp.

With these two symbols being exported in the ELF file, MATLAB will run your code. Note, however,
that the content written to stdout does not appear in MATLAB's console. This possibly is an
artefact of us not relying on MATHWORK's tweaked C++ standard library.

Written by
  Alessandro Motta <alessandro.motta@brain.mpg.de>
*/

#[macro_export]
macro_rules! mex_function {
    ($_nlhs:ident, $_plhs:ident, $_nrhs:ident, $_prhs:ident, $_body:block) => {

#[no_mangle]
pub unsafe extern fn mexfilerequiredapiversion(
    built_by_rel: *mut c_uint,
    target_api_ver: *mut c_uint
) {
    *built_by_rel = 0x2016b;
    *target_api_ver = 0x07300000;
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern fn mexFunction(
    nlhs: c_int, plhs: *mut MxArrayMut,
    nrhs: c_int, prhs: *const MxArray
) {
    unsafe fn body(
        $_nlhs: c_int, $_plhs: *mut MxArrayMut,
        $_nrhs: c_int, $_prhs: *const MxArray) -> Result<()> $_body

    match unsafe { body(nlhs, plhs, nrhs, prhs) } {
        Ok(_) => (),
        Err(msg) => die(msg)
    }
}
    }
}
