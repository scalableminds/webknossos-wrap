/* wkwrap.h
 * A header-only C++ library for reading, writing and
 * compressing wk-wkrap files.
 *
 * Written by
 * Alessandro Motta <alessandro.motta@brain.mpg.de>
 */

#ifndef WKWRAP_H
#define WKWRAP_H

#include <fcntl.h> /* for open */
#include <algorithm> /* for std::min */
#include <assert.h> /* for assert */
#include <stdint.h> /* for uintX_T */
#include <stdio.h> /* for fopen, etc. */
#include <string.h> /* for memcpy */
#include <unistd.h> /* for ftruncate */

/* Morton */
#include <morton.h>

/* LZ4 */
#include <lz4.h>
#include <lz4hc.h>

/* This library was developed with version 1.7.4 of LZ4, which
 * contains a LZ4HC_DEFAULT_CLEVEL constant. This constant is
 * however missing in version 1.7.1, which is on GABA. */
#ifndef LZ4HC_DEFAULT_CLEVEL
#define LZ4HC_DEFAULT_CLEVEL 9
#endif

/* every wk-wrap file begins with these magic bytes */
const uint8_t headerMagic[] = {'W', 'K', 'W'};

/* CLEN stands for cube length, and
 * BLEN stands for length in blocks */
#define BLOCK_CLEN_LOG2 5
#define BLOCK_CLEN (1 << BLOCK_CLEN_LOG2)
#define BLOCK_NUMEL (1 << (3 * BLOCK_CLEN_LOG2))

#define FILE_CLEN_LOG2 10
#define FILE_BLEN_LOG2 (FILE_CLEN_LOG2 - BLOCK_CLEN_LOG2)
#define FILE_CLEN (1 << FILE_CLEN_LOG2)
#define FILE_NUMEL (1 << (3 * FILE_CLEN_LOG2))

#define HI_NIBBLE(x) (((x) & 0xF0) >> 4)
#define LO_NIBBLE(x)  ((x) & 0x0F)

typedef struct {
  uint8_t  magic[sizeof(headerMagic)];
  uint8_t  version;
  uint8_t  lensLog2;
  uint8_t  blockType;
  uint8_t  voxelType;
  uint8_t  voxelSize;
  uint64_t dataOffset;
} header_t;

typedef enum {
  VOXEL_TYPE_INVALID,
  VOXEL_TYPE_UINT8,
  VOXEL_TYPE_UINT16,
  VOXEL_TYPE_UINT32,
  VOXEL_TYPE_UINT64,
  VOXEL_TYPE_FLOAT,
  VOXEL_TYPE_DOUBLE,
  VOXEL_TYPE_UNKNOWN
} voxelType_t;

typedef enum {
  BLOCK_TYPE_INVALID,
  BLOCK_TYPE_RAW,
  BLOCK_TYPE_LZ4,
  BLOCK_TYPE_LZ4HC,
  BLOCK_TYPE_UNKNOWN
} blockType_t;

/* helpers to convert types into their voxelType_t */
template<typename T> uint8_t wkwGetVoxelType();
template<> uint8_t wkwGetVoxelType<uint8_t>(){ return VOXEL_TYPE_UINT8; }
template<> uint8_t wkwGetVoxelType<uint16_t>(){ return VOXEL_TYPE_UINT16; }
template<> uint8_t wkwGetVoxelType<uint32_t>(){ return VOXEL_TYPE_UINT32; }
template<> uint8_t wkwGetVoxelType<uint64_t>(){ return VOXEL_TYPE_UINT64; }
template<> uint8_t wkwGetVoxelType<float>(){ return VOXEL_TYPE_FLOAT; }
template<> uint8_t wkwGetVoxelType<double>(){ return VOXEL_TYPE_DOUBLE; }

int8_t wkwLog2(uint64_t val) {
  /* make sure it's not zero */
  if(val == 0) return -1;

  /* make sure it's a power of two */
  if(val & (val - 1)) return -2;

  /* valid value */
  int8_t ret = 0;
  while(val > 1){
      val >>= 1;
      ret++;
  }

  return ret;
}

int wkwReadHeader(FILE * in, header_t * h){
  if(fread(h, sizeof(header_t), 1, in) != 1) return -1;
  return 0;
}

int wkwCheckHeader(header_t * h){
  if(memcmp(h->magic, headerMagic, sizeof(headerMagic)) != 0) return -1; /* magic */
  if(h->version == 0) return -2; /* version */
  if(h->voxelType == 0) return -3; /* voxel type */
  if(h->blockType == 0) return -4; /* block type */

  /* The following conditions do not need to be met by a valid wk-wrap file.
   * But in its current version, this library cannot handle the more general case. */
  if(HI_NIBBLE(h->lensLog2) != FILE_BLEN_LOG2) return -5;
  if(LO_NIBBLE(h->lensLog2) != BLOCK_CLEN_LOG2) return -6;

  /* This probably needs to be changed for the header file. */
  if(h->dataOffset < sizeof(header_t)) return -7;

  return 0;
}

template<typename T>
int wkwCompressBlocks(
    uint64_t jumpEntry,
    uint64_t jumpTable[],
    FILE * in, FILE * out)
{
  T rawBuf[BLOCK_NUMEL];
  uint8_t encBuf[LZ4_COMPRESSBOUND(sizeof(T) * BLOCK_NUMEL)];

  const size_t blockCount = FILE_NUMEL / BLOCK_NUMEL;
  for(size_t blockIdx = 0; blockIdx < blockCount; ++blockIdx){
    size_t rawLen, encLen;

    /* read block from input file */
    assert((rawLen = fread(rawBuf, sizeof(T), BLOCK_NUMEL, in)) == BLOCK_NUMEL);

    /* compress block */
    assert((encLen = LZ4_compress_HC(
      reinterpret_cast<const char *>(rawBuf), reinterpret_cast<char *>(encBuf),
      sizeof(T) * rawLen, sizeof(encBuf), LZ4HC_DEFAULT_CLEVEL)) != 0);

    /* write compressed block */
    assert(fwrite((const void *) encBuf, 1, encLen, out) == encLen);

    /* update jump table */
    jumpEntry += encLen;
    jumpTable[blockIdx] = jumpEntry;
  }

  return 0;
}

int wkwCompress(const char * inFile, const char * outFile){
  int err = 0;
  FILE * in, * out;
  header_t inHeader, outHeader;

  /* prepare jump table, etc. */
  uint64_t jumpTable[FILE_NUMEL / BLOCK_NUMEL];
  uint64_t dataOffset = sizeof(header_t) + sizeof(jumpTable);

  /* open files */
  if((in = fopen(inFile, "rb")) == NULL && (err = -1)) goto cleanup;
  if((out = fopen(outFile, "wb")) == NULL && (err = -2)) goto cleanup;

  /* read and validate header of input file */
  if(wkwReadHeader(in, &inHeader) != 0 && (err = -3)) goto cleanup;
  if(wkwCheckHeader(&inHeader) != 0 && (err = -4)) goto cleanup;
  if(inHeader.blockType != BLOCK_TYPE_RAW && (err = -5)) goto cleanup;

  /* prepare data streams */
  if(fseek(in, inHeader.dataOffset, SEEK_SET) != 0 && (err = -6)) goto cleanup;
  if(fseek(out, dataOffset, SEEK_SET) != 0 && (err = -7)) goto cleanup;

  /* actually do the thing */
  switch(inHeader.voxelType){
    case VOXEL_TYPE_UINT8:
      err = wkwCompressBlocks<uint8_t> (dataOffset, jumpTable, in, out); break;
    case VOXEL_TYPE_UINT16:
      err = wkwCompressBlocks<uint16_t>(dataOffset, jumpTable, in, out); break;
    case VOXEL_TYPE_UINT32:
      err = wkwCompressBlocks<uint32_t>(dataOffset, jumpTable, in, out); break;
    case VOXEL_TYPE_UINT64:
      err = wkwCompressBlocks<uint64_t>(dataOffset, jumpTable, in, out); break;
    case VOXEL_TYPE_FLOAT:
      err = wkwCompressBlocks<float>   (dataOffset, jumpTable, in, out); break;
    case VOXEL_TYPE_DOUBLE:
      err = wkwCompressBlocks<double>  (dataOffset, jumpTable, in, out); break;

    /* if this ever happens, the header validation failed miserably */
    default: assert(0);
  }
  /* just to be future proof */
  if(err && (err = -8)) goto cleanup;

  /* build header of output file */
  outHeader = inHeader;
  outHeader.blockType = BLOCK_TYPE_LZ4HC;
  outHeader.dataOffset = dataOffset;

  /* write header and jump table */
  assert(fseek(out, 0, SEEK_SET) == 0);
  assert(fwrite((const void *) &outHeader, sizeof(header_t), 1, out) == 1);
  assert(fwrite((const void *) jumpTable, sizeof(jumpTable), 1, out) == 1);

cleanup:
  if(in != NULL) fclose(in);
  if(out != NULL) fclose(out);

  return err;
}

template<typename T>
inline T * wkwGetBlkPointer(
  T * in,
  const size_t inClenLog2,
  const size_t blkIdx)
{
  /* calculate position of loaded cube with respect
   * to the entire requested data cube */
  uint_fast16_t curBlkX, curBlkY, curBlkZ;
  morton3D_32_decode(blkIdx, curBlkX, curBlkY, curBlkZ);

  return &in[
    (curBlkX <<  BLOCK_CLEN_LOG2) +
    (curBlkY << (BLOCK_CLEN_LOG2  +  inClenLog2)) +
    (curBlkZ << (BLOCK_CLEN_LOG2  + (inClenLog2 << 1)))];
}

template<typename T>
inline void wkwCopyBlk(
    const T * in, const size_t inClenLog2,
    T * out, const size_t outClenLog2)
{

  for(size_t curZ = 0; curZ < BLOCK_CLEN; ++curZ){
    const T * curIn = &in[curZ << (2 * inClenLog2)];
    T * curOut = &out[curZ << (2 * outClenLog2)];

    /* copy Fortran-order stripes */
    for(size_t curY = 0; curY < BLOCK_CLEN; ++curY){
      memcpy(curOut, curIn, sizeof(T) << BLOCK_CLEN_LOG2);

      /* continue */
      curIn = &curIn[1 << inClenLog2];
      curOut = &curOut[1 << outClenLog2];
    }
  }
}

template<typename T>
int wkwReadRaw(
    FILE * in,
    const size_t blkIdx,
    const size_t outClen,
    T * out)
{
  /* validate block index */
  if(blkIdx >= FILE_NUMEL / BLOCK_NUMEL) return -1;

  /* validate cube side length */
  const int8_t outClenLog2 = wkwLog2(outClen);
  if(outClenLog2 < BLOCK_CLEN_LOG2) return -2;

  /* seek to offset */
  const size_t offBytes = sizeof(header_t) + sizeof(T) * BLOCK_NUMEL * blkIdx;
  assert(fseek(in, offBytes, SEEK_SET) == 0);

  /* prepare */
  T buf[BLOCK_NUMEL];
  const size_t blkCount = 1 << (3 * (outClenLog2 - BLOCK_CLEN_LOG2));

  for(size_t curBlkIdx = 0; curBlkIdx < blkCount; ++curBlkIdx){
    /* read one block worth of data */
    assert(fread(buf, sizeof(T), BLOCK_NUMEL, in) == BLOCK_NUMEL);

    /* copy buffer to putput */
    T * curOut = wkwGetBlkPointer<T>(out, outClenLog2, curBlkIdx);
    wkwCopyBlk<T>(buf, BLOCK_CLEN_LOG2, curOut, outClenLog2);
  }

  return 0;
}

template<typename T>
int wkwReadLZ4(
    FILE * in,
    const size_t blkIdx,
    const size_t outClen,
    T * out)
{
  /* validate block index */
  if(blkIdx >= FILE_NUMEL / BLOCK_NUMEL) return -1;

  /* validate cube side length */
  const int8_t outClenLog2 = wkwLog2(outClen);
  if(outClenLog2 < BLOCK_CLEN_LOG2) return -2;

  /* read jump table */
  size_t blkCount = 1 << (3 * (outClenLog2 - BLOCK_CLEN_LOG2));
  uint64_t jumpTable[blkCount + 1];

  /* go to first relevant entry of jump table */
  size_t jumpOff = blkIdx * sizeof(uint64_t)  /* relative position to dataOffset */
       + sizeof(header_t) - sizeof(uint64_t); /* position of dataOffset field */

  assert(fseek(in, jumpOff, SEEK_SET) == 0);
  assert(fread(jumpTable, sizeof(uint64_t), blkCount + 1, in) == blkCount + 1);

  /* seek to first compressed block */
  assert(fseek(in, jumpTable[0], SEEK_SET) == 0);

  /* determine buffer size */
  T encBuf[BLOCK_NUMEL];
  T rawBuf[BLOCK_NUMEL];

  for(size_t curBlkIdx = 0; curBlkIdx < blkCount; ++curBlkIdx){
    /* read compressed Fortran-order block */
    size_t toRead = jumpTable[curBlkIdx + 1] - jumpTable[curBlkIdx];
    assert(fread(encBuf, 1, toRead, in) == toRead);

    /* decompress block */
    assert(LZ4_decompress_safe(
      (const char *) encBuf, (char *) rawBuf, toRead, sizeof(rawBuf)) >= 0);

    /* write to output */
    T * curOut = wkwGetBlkPointer<T>(out, outClenLog2, curBlkIdx);
    wkwCopyBlk<T>(rawBuf, BLOCK_CLEN_LOG2, curOut, outClenLog2);
  }

  return 0;
}

/* wkwRead
 *   Reads a cube of voxel data from disk.
 *
 * Type parameter
 *   T:        Type of voxel data
 *
 * Function arguments
 *   fileName: Absolute path to wk-wrap file
 *   offVec:   X, Y and Z offset of the cube.
 *             Each entry must be an integer multiple of clen.
 *   clen:     Side length of the desired data cube.
 *             Must be a power of two and at least as large as BLOCK_CLEN.
 *   out:      Destination buffer. Must be allocated by caller.
 *
 * Return value
 *     0       if function call succeeded
 *   < 0       if function call failed
 */
template<typename T>
int wkwRead(
    const char * fileName,
    const size_t offVec[3],
    const size_t clen,
    T * out)
{
  /* state */
  int err = 0;
  FILE * in = NULL;

  /* validate cube length */
  const int8_t clenLog2 = wkwLog2(clen);
  if(clenLog2 < 0) return -1;
  if(clenLog2 < BLOCK_CLEN_LOG2) return -1;

  /* validate offset */
  if(offVec[0] % clen || offVec[1] % clen || offVec[2] % clen) return -2;
  const size_t blkIdx = morton3D_32_encode(
    offVec[0] >> BLOCK_CLEN_LOG2, offVec[1] >> BLOCK_CLEN_LOG2, offVec[2] >> BLOCK_CLEN_LOG2);

  /* open file */
  if((in = fopen(fileName, "rb")) == NULL) return -3;

  /* read and validate header */
  header_t header;
  if(wkwReadHeader(in, &header) && (err = -4)) goto cleanup;
  if(wkwCheckHeader(&header) && (err = -5)) goto cleanup;
  if(header.voxelType != wkwGetVoxelType<T>() && (err = -6)) goto cleanup;
  if(header.voxelSize != sizeof(T) && (err = -7)) goto cleanup;

  switch(header.blockType){
    case BLOCK_TYPE_RAW:
      err = wkwReadRaw(in, blkIdx, clen, out);
      break;
    case BLOCK_TYPE_LZ4:
    case BLOCK_TYPE_LZ4HC:
      err = wkwReadLZ4(in, blkIdx, clen, out);
      break;

    /* this should never happen */
    default: assert(0);
  }

  /* to be future proof */
  if(err && (err -= 7)) goto cleanup;

cleanup:
  if(in != NULL) fclose(in);
  return err;
}

template<typename T>
int wkwWriteRaw(
    const char * fileName,
    const size_t offVec[3],
    const size_t clen,
    const T * in)
{
  /* state */
  int err = 0;
  header_t header;
  FILE * out = NULL;

  /* validate size*/
  const int8_t clenLog2 = wkwLog2(clen);
  if(clenLog2 < BLOCK_CLEN_LOG2) return -2;

  /* determine number of blocks to read */
  const size_t blkCount = 1 << (3 * (clenLog2 - BLOCK_CLEN_LOG2));

  /* validate offset */
  if(offVec[0] % clen || offVec[1] % clen || offVec[2] % clen) return -3;
  if(offVec[0] > FILE_CLEN || offVec[1] > FILE_CLEN || offVec[2] > FILE_CLEN) return -3;

  /* determine where we start to read */
  size_t blkIdx = morton3D_32_encode(
    offVec[0] >> BLOCK_CLEN_LOG2, offVec[1] >> BLOCK_CLEN_LOG2, offVec[2] >> BLOCK_CLEN_LOG2);
  size_t offsetBytes = sizeof(header_t) + sizeof(T) * BLOCK_NUMEL * blkIdx;

  int outFd;
  int outFdFlags = O_RDWR | O_CREAT;
  mode_t outFdMode = S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH;

  /* create file descriptor first and file stream second */
  assert((outFd = open(fileName, outFdFlags, outFdMode)) != -1);
  assert((out = fdopen(outFd, "rb+")) != NULL);

  /* check if file is a pre-existing wk-wrap file */
  if(!wkwReadHeader(out, &header) && !wkwCheckHeader(&header)){
    /* indeed, it is */
    if(header.blockType != BLOCK_TYPE_RAW && (err = -4)) goto cleanup;
    if(header.voxelType != wkwGetVoxelType<T>() && (err = -5)) goto cleanup;
    if(header.voxelSize != sizeof(T) && (err = -6)) goto cleanup;
  }else{
    /* build header */
    memcpy(header.magic, headerMagic, sizeof(headerMagic));
    header.version    = 1;
    header.lensLog2   = FILE_BLEN_LOG2 << 4;
    header.lensLog2  |= BLOCK_CLEN_LOG2;
    header.blockType  = BLOCK_TYPE_RAW;
    header.voxelType  = wkwGetVoxelType<T>();
    header.voxelSize  = sizeof(T);
    header.dataOffset = sizeof(header_t);

    /* write header to file */
    assert(fseek(out, 0, SEEK_SET) == 0);
    assert(fwrite((const void *) &header, sizeof(header_t), 1, out) == 1);

    /* truncate file to correct size */
    const size_t fileSize = sizeof(header_t) + FILE_NUMEL * sizeof(T);
    assert(ftruncate(fileno(out), fileSize) == 0);
  }

  /* seek to beginning of block */
  assert(fseek(out, offsetBytes, SEEK_SET) == 0);

  /* prepare variables */
  T buf[BLOCK_NUMEL];

  /* iterate over Fortran-order blocks */
  for(size_t curBlkIdx = 0; curBlkIdx < blkCount; ++curBlkIdx){
    /* copy Fortran-order block to buffer */
    const T * curIn = wkwGetBlkPointer<const T>(in, clenLog2, curBlkIdx);
    wkwCopyBlk<T>(curIn, clenLog2, buf, BLOCK_CLEN_LOG2);

    /* write current buffer to file */
    assert(fwrite(buf, sizeof(T), BLOCK_NUMEL, out) == BLOCK_NUMEL);
  }

cleanup:
  /* cleaning up */
  if(out) fclose(out);
  return err;
}

#endif /* WKWRAP_H */