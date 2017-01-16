/* barrelSave.cpp
 * Encodes a three-dimensional Fortran-order matrix to Morton order
 * and writes the result to a file.
 *
 * Written by
 * Alessandro Motta <alessandro.motta@brain.mpg.de>
 */

#include <barrel.h>
#include <stdint.h>

#include "mex.h"
#include "matrix.h"

void mexFunction(
    int nlhs, mxArray * plhs[],
    int nrhs, const mxArray * prhs[])
{
  if(nrhs != 3)
    mexErrMsgTxt("Invalid number of input arguments");

  const mxArray * fileNameArr = prhs[0];
  const mxArray * inArr = prhs[1];
  const mxArray * offArr = prhs[2];

  /* check inputs */
  if(!mxIsChar(fileNameArr))
    mexErrMsgTxt("First input must be a file path");
  const char * fileName = mxArrayToString(fileNameArr);

  if(!mxIsNumeric(inArr) || mxGetNumberOfDimensions(inArr) != 3 || mxIsComplex(inArr))
    mexErrMsgTxt("Data must be a numeric three-dimensional matrix");

  const mxClassID inClass = mxGetClassID(inArr);
  const mwSize * inSz = mxGetDimensions(inArr);

  /* make sure it's a cube */
  if(inSz[0] != inSz[1] || inSz[0] != inSz[2] || inSz[1] != inSz[2])
    mexErrMsgTxt("Data must be a N x N x N matrix");
  const size_t inSize = (size_t) inSz[0];

  if(!mxIsDouble(offArr) || mxGetNumberOfElements(offArr) != 3 || mxIsComplex(offArr))
    mexErrMsgTxt("Offset must be a vector of three doubles");

  const double * offPr = mxGetPr(offArr);
  const size_t offVec[] = {
    (size_t) offPr[0] - 1,
    (size_t) offPr[1] - 1,
    (size_t) offPr[2] - 1};

  /* state */
  int errorCode = 0;

  /* actually write file */
  switch(inClass){
    case mxUINT8_CLASS:
      errorCode = barrelWriteRaw<uint8_t>(
        fileName, offVec, inSize, (const uint8_t *) mxGetData(inArr));
      break;
    case mxUINT32_CLASS:
      errorCode = barrelWriteRaw<uint32_t>(
        fileName, offVec, inSize, (const uint32_t *) mxGetData(inArr));
      break;
    case mxSINGLE_CLASS:
      errorCode = barrelWriteRaw<float>(
        fileName, offVec, inSize, (const float *) mxGetData(inArr));
      break;
    default:
      mexErrMsgTxt("Class of input argument is not supported");
      break;
  }

  if(errorCode)
    mexErrMsgTxt("An error occured while saving file");
}
