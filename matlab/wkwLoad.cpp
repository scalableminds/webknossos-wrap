/* barrelLoad.cpp
 * Loads data from a Morton-encoded file and decodes it to
 * Fortran order on-the-fly.
 *
 * Written by
 * Alessandro Motta <alessandro.motta@brain.mpg.de>
 */

#include "mex.h"
#include "matrix.h"

#include <wkwrap.h>
#include <iostream>

void mexFunction(
    int nlhs, mxArray * plhs[],
    int nrhs, const mxArray * prhs[])
{
    if(nrhs != 4)
      mexErrMsgTxt("Invalid number of input arguments");

    const mxArray * fileNameArr = prhs[0];
    const mxArray * sizeArr = prhs[1];
    const mxArray * offArr = prhs[2];
    const mxArray * typeArr = prhs[3];

    /* check inputs */
    if(!mxIsChar(fileNameArr))
      mexErrMsgTxt("First input must be a file path");
    const char * fileName = mxArrayToString(fileNameArr);

    if(!mxIsScalar(sizeArr) || !mxIsDouble(sizeArr) || mxIsComplex(sizeArr))
      mexErrMsgTxt("Size input must be a double");
    const size_t size = (size_t) mxGetScalar(sizeArr);

    if(!mxIsDouble(offArr) || mxGetNumberOfElements(offArr) != 3 || mxIsComplex(offArr))
      mexErrMsgTxt("Offset must be a vector of three doubles");

    const double * offPr = mxGetPr(offArr);
    const size_t offVec[] = {
      (size_t) offPr[0] - 1,
      (size_t) offPr[1] - 1,
      (size_t) offPr[2] - 1};

    if(!mxIsChar(typeArr))
      mexErrMsgTxt("Data type must be a string");
    const char * type = mxArrayToString(typeArr);

    /* allocate output */
    int errorCode = 0;
    mxArray * out = NULL;
    mwSize outSize[] = {size, size, size};

    if(!strcmp(type, "uint8")){
      out = mxCreateUninitNumericArray(3, outSize, mxUINT8_CLASS, mxREAL);
      errorCode = wkwRead<uint8_t>(fileName, offVec, size, (uint8_t *) mxGetData(out));
    }else if(!strcmp(type, "uint32")){
      out = mxCreateUninitNumericArray(3, outSize, mxUINT32_CLASS, mxREAL);
      errorCode = wkwRead<uint32_t>(fileName, offVec, size, (uint32_t *) mxGetData(out));
    }else if(!strcmp(type, "single")){
      out = mxCreateUninitNumericArray(3, outSize, mxSINGLE_CLASS, mxREAL);
      errorCode = wkwRead<float>(fileName, offVec, size, (float *) mxGetData(out));
    }else{
      mexErrMsgTxt("Given data type is not supported");
    }

    if(out){
      if(errorCode){
        std::cout << "Error: " << errorCode << std::endl;
      }else{
        /* set output */
        plhs[0] = out;
      }
    }
}
