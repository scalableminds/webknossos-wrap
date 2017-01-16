/* barrelCompress.cpp
 * Compresses a raw barrel file using LZ4.
 *
 * Written by
 * Alessandro Motta <alessandro.motta@brain.mpg.de>
 */

#include "mex.h"
#include <wkwrap.h>

void mexFunction(
    int nlhs, mxArray * plhs[],
    int nrhs, const mxArray * prhs[])
{
  if(nrhs != 2)
    mexErrMsgTxt("Invalid number of input arguments");

  const mxArray * inFileArr = prhs[0];
  const mxArray * outFileArr = prhs[1];

  /* check inputs */
  if(!mxIsChar(inFileArr) || !mxIsChar(outFileArr))
    mexErrMsgTxt("Inputs must be character arrays");

  const char * inFile = mxArrayToString(inFileArr);
  const char * outFile = mxArrayToString(outFileArr);

  if(wkwCompress(inFile, outFile))
    mexErrMsgTxt("Error while compressing file");
}
