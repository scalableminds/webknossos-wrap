/* barrel.h
 * A header-only C++ library for reading, writing and
 * compressing barrel files.
 *
 * Written by
 * Alessandro Motta <alessandro.motta@brain.mpg.de>
 */

#ifndef BARREL_H
#define BARREL_H

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

/* every barrel file begins with these magic bytes */
const uint8_t headerMagic[] = {'M', 'P', 'I', 'B', 'R'};

/* CLEN stands for cube length */
#define FILE_CLEN_LOG2 10
#define FILE_CLEN (1 << FILE_CLEN_LOG2)
#define FILE_NUMEL (1 << (3 * FILE_CLEN_LOG2))

#define BLOCK_CLEN_LOG2 5
#define BLOCK_CLEN (1 << BLOCK_CLEN_LOG2)
#define BLOCK_NUMEL (1 << (3 * BLOCK_CLEN_LOG2))

typedef struct {
  uint8_t magic[sizeof(headerMagic)];
  uint8_t version;
  uint8_t dataType;
  uint8_t blockType;
} header_t;

typedef enum {
  DATA_TYPE_INVALID,
  DATA_TYPE_UINT8,
  DATA_TYPE_UINT16,
  DATA_TYPE_UINT32,
  DATA_TYPE_UINT64,
  DATA_TYPE_FLOAT,
  DATA_TYPE_DOUBLE,
  DATA_TYPE_UNKNOWN
} dataType_t;

typedef enum {
  BLOCK_TYPE_INVALID,
  BLOCK_TYPE_RAW,
  BLOCK_TYPE_LZ4_32C,
  BLOCK_TYPE_LZ4HC_32C,
  BLOCK_TYPE_UNKNOWN
} blockType_t;

/* helpers to convert types into their dataType_t */
template<typename T> uint8_t barrelGetDataType();
template<> uint8_t barrelGetDataType<uint8_t>(){ return DATA_TYPE_UINT8; }
template<> uint8_t barrelGetDataType<uint16_t>(){ return DATA_TYPE_UINT16; }
template<> uint8_t barrelGetDataType<uint32_t>(){ return DATA_TYPE_UINT32; }
template<> uint8_t barrelGetDataType<uint64_t>(){ return DATA_TYPE_UINT64; }
template<> uint8_t barrelGetDataType<float>(){ return DATA_TYPE_FLOAT; }
template<> uint8_t barrelGetDataType<double>(){ return DATA_TYPE_DOUBLE; }

int8_t barrelLog2(uint64_t val) {
  /* make sure its not zero */
  if(val == 0) return -1;

  /* make sure its a power of two */
  if(val & (val - 1)) return -2;

  /* valid value */
  int8_t ret = 0;
  while(val > 1){
      val >>= 1;
      ret++;
  }

  return ret;
}

int barrelReadHeader(FILE * in, header_t * h){
  if(fread(h, sizeof(header_t), 1, in) != 1) return -1;
  return 0;
}

int barrelCheckHeader(header_t * h){
  if(memcmp(h->magic, headerMagic, sizeof(headerMagic)) != 0) return -1;
  if(h->version < 1 || h->version > 1) return -2;
  if(!h->dataType || h->dataType >= DATA_TYPE_UNKNOWN) return -3;
  if(!h->blockType || h->blockType >= BLOCK_TYPE_UNKNOWN) return -4;
  return 0;
}

template<typename T>
int barrelCompressBlocks(FILE * in, FILE * out){
  T rawBuf[BLOCK_NUMEL];
  uint8_t encBuf[LZ4_COMPRESSBOUND(sizeof(T) * BLOCK_NUMEL)];

  /* jump table */
  uint64_t jumpEntry = 0;
  uint64_t jumpTable[FILE_NUMEL / BLOCK_NUMEL];

  /* remember where to place the jump table */
  off_t jumpTableOff = ftell(out);

  /* jump to beginning of data segment */
  off_t encDataOff = sizeof(header_t) + sizeof(jumpTable);
  assert(fseek(out, encDataOff, SEEK_SET) == 0);

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

  /* write jump table */
  assert(fseek(out, jumpTableOff, SEEK_SET) == 0);
  assert(fwrite((const void *) jumpTable, sizeof(jumpTable), 1, out) == 1);

  return 0;
}

int barrelCompress(const char * inFile, const char * outFile){
  int err = 0;
  FILE * in, * out;
  header_t inHeader, outHeader;

  /* open files */
  if((in = fopen(inFile, "rb")) == NULL && (err = -1)) goto cleanup;
  if((out = fopen(outFile, "wb")) == NULL && (err = -2)) goto cleanup;

  /* read and validate header of input file */
  if(barrelReadHeader(in, &inHeader) != 0 && (err = -3)) goto cleanup;
  if(barrelCheckHeader(&inHeader) != 0 && (err = -4)) goto cleanup;

  /* build and write header of output file */
  outHeader = inHeader;
  outHeader.blockType = BLOCK_TYPE_LZ4HC_32C;
  assert(fwrite((const void *) &outHeader, sizeof(header_t), 1, out) == 1);
  assert(fflush(out) == 0);

  /* actually do the thing */
  switch(inHeader.dataType){
    case DATA_TYPE_UINT8:  err = barrelCompressBlocks<uint8_t> (in, out); break;
    case DATA_TYPE_UINT16: err = barrelCompressBlocks<uint16_t>(in, out); break;
    case DATA_TYPE_UINT32: err = barrelCompressBlocks<uint32_t>(in, out); break;
    case DATA_TYPE_UINT64: err = barrelCompressBlocks<uint64_t>(in, out); break;
    case DATA_TYPE_FLOAT:  err = barrelCompressBlocks<float>   (in, out); break;
    case DATA_TYPE_DOUBLE: err = barrelCompressBlocks<double>  (in, out); break;

    /* if this ever happens, the header validation failed miserably */
    default: assert(0);
  }

  /* just to be future proof */
  if(err && (err -= 4)) goto cleanup;

cleanup:
  if(in != NULL) fclose(in);
  if(out != NULL) fclose(out);

  return err;
}

template<typename T>
inline T * barrelGetBlkPointer(
  const T * in,
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
inline void barrelCopyBlk(
    const T * in, const size_t inClen,
    T * out, const size_t outClen)
{
  for(size_t curIdx = 0; curIdx < BLOCK_CLEN * BLOCK_CLEN; ++curIdx){
    /* copy Fortran-order stripe */
    memcpy((void *) in, (const void *) out, sizeof(T) * BLOCK_CLEN);

    /* move pointers forward */
    in += inClen;
    out += outClen;
  }
}

template<typename T>
int barrelReadRaw(
    FILE * in,
    const size_t blkIdx,
    const size_t outClen,
    T * out)
{
  /* validate block index */
  if(blkIdx >= FILE_NUMEL / BLOCK_NUMEL) return -1;

  /* validate cube side length */
  const int8_t outClenLog2 = barrelLog2(outClen);
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
    T * curOut = barrelGetBlkPointer<T>(out, outClenLog2, curBlkIdx);
    barrelCopyBlk<T>(buf, BLOCK_CLEN, curOut, outClen);
  }

  return 0;
}

template<typename T>
int barrelReadLZ4(
    FILE * in,
    const size_t blkIdx,
    const size_t outClen,
    T * out)
{
  /* validate block index */
  if(blkIdx >= FILE_NUMEL / BLOCK_NUMEL) return -1;

  /* validate cube side length */
  const int8_t outClenLog2 = barrelLog2(outClen);
  if(outClenLog2 < BLOCK_CLEN_LOG2) return -2;

  /* read jump table */
  size_t blkCount = 1 << (3 * (outClenLog2 - BLOCK_CLEN_LOG2));
  uint64_t jumpTable[blkCount + 1];

  if(blkIdx){
    /* skip a couple of blocks */
    size_t jumpOff = sizeof(header_t) + (blkIdx - 1) * sizeof(uint64_t);
    assert(fseek(in, jumpOff, SEEK_SET) == 0);
    assert(fread(jumpTable, sizeof(uint64_t), blkCount + 1, in) == blkCount + 1);
  }else{
    /* start with first block */
    jumpTable[0] = 0;
    assert(fread(&jumpTable[1], sizeof(uint64_t), blkCount, in) == blkCount);
  }

  /* seek to offset */
  const size_t offBytes =
    jumpTable[0] + sizeof(header_t) +
    sizeof(uint64_t) * FILE_NUMEL / BLOCK_NUMEL;
  assert(fseek(in, offBytes, SEEK_SET) == 0);

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
    T * curOut = barrelGetBlkPointer(out, outClenLog2, curBlkIdx);
    barrelCopyBlk(rawBuf, BLOCK_CLEN, out, outClen);
  }

  return 0;
}

template<typename T>
int barrelRead(
    const char * fileName,
    const size_t offVec[3],
    const size_t clen,
    T * out)
{
  /* state */
  int err = 0;
  FILE * in = NULL;

  /* validate cube length */
  const int8_t clenLog2 = barrelLog2(clen);
  if(clenLog2 < 0) return -1;
  if(clenLog2 < BLOCK_CLEN_LOG2) return -1;

  /* validate offset */
  if(offVec[0] % clen || offVec[1] % clen || offVec[2] % clen) return -2;
  const size_t blockIdx = morton3D_32_encode(
    offVec[0] >> BLOCK_CLEN_LOG2,
    offVec[1] >> BLOCK_CLEN_LOG2,
    offVec[1] >> BLOCK_CLEN_LOG2);

  /* open file */
  if((in = fopen(fileName, "rb")) == NULL) return -3;

  /* read and validate header */
  header_t header;
  if(barrelReadHeader(in, &header) && (err = -4)) goto cleanup;
  if(barrelCheckHeader(&header) && (err = -5)) goto cleanup;
  if(barrelGetDataType<T>() != header.dataType && (err = -6)) goto cleanup;

  switch(header.blockType){
    case BLOCK_TYPE_RAW:
      err = barrelReadRaw(in, blockIdx, clen, out);
      break;
    case BLOCK_TYPE_LZ4_32C:
    case BLOCK_TYPE_LZ4HC_32C:
      err = barrelReadLZ4(in, blockIdx, clen, out);
      break;

    /* this should never happen */
    default: assert(0);
  }

  /* to be future proof */
  if(err && (err -= 6)) goto cleanup;

cleanup:
  if(in != NULL) fclose(in);
  return err;
}

template<typename T>
int barrelWriteRaw(
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
  const int8_t clenLog2 = barrelLog2(clen);
  if(clenLog2 < BLOCK_CLEN_LOG2) return -2;

  /* validate offset */
  if(offVec[0] % clen || offVec[1] % clen || offVec[2] % clen) return -3;
  if(offVec[0] > FILE_CLEN || offVec[1] > FILE_CLEN || offVec[2] > FILE_CLEN) return -3;

  int outFd;
  int outFdFlags = O_RDWR | O_CREAT;
  mode_t outFdMode = S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH;

  /* create file descriptor first and file stream second */
  assert((outFd = open(fileName, outFdFlags, outFdMode)) != -1);
  assert((out = fdopen(outFd, "rb+")) != NULL);

  /* check if file is empty
   * if so, let's write a header */
  if(ftell(out) == 0){
    /* build header */
    memcpy(header.magic, headerMagic, sizeof(headerMagic));
    header.version = 1;
    header.dataType = barrelGetDataType<T>();
    header.blockType = BLOCK_TYPE_RAW;

    /* write header to file */
    assert(fwrite((const void *) &header, sizeof(header_t), 1, out) == 1);

    /* truncate file to correct size */
    const size_t fileSize = sizeof(header_t) + FILE_NUMEL * sizeof(T);
    assert(ftruncate(fileno(out), fileSize) == 0);
  }else{
    /* read header */
    assert(fseek(out, 0, SEEK_SET) == 0);
    assert(barrelReadHeader(out, &header) == 0);

    /* validate header */
    if(barrelCheckHeader(&header) != 0 && (err = -5)) goto cleanup;
    if(barrelGetDataType<T>() != header.dataType && (err = -6)) goto cleanup;
  }

  /* seek to beginning of block */
  size_t blkIdx = morton3D_32_encode(
    offVec[0] >> BLOCK_CLEN_LOG2,
    offVec[1] >> BLOCK_CLEN_LOG2,
    offVec[2] >> BLOCK_CLEN_LOG2);
  off_t offsetBytes = sizeof(header_t) + blkIdx * BLOCK_NUMEL * sizeof(T);
  assert(fseek(out, offsetBytes, SEEK_SET) == 0);

  /* prepare variables */
  T buf[BLOCK_NUMEL];
  const size_t blkCount = 1 << (3 * (clenLog2 - BLOCK_CLEN_LOG2));

  /* iterate over Fortran-order blocks */
  for(size_t curBlkIdx = 0; curBlkIdx < blkCount; ++curBlkIdx){
    /* copy Fortran-order block to buffer */
    const T * curIn = barrelGetBlkPointer(in, clenLog2, curBlkIdx);
    barrelCopyBlk(curIn, clen, buf, BLOCK_CLEN);

    /* write current buffer to file */
    assert(fwrite(buf, sizeof(T), BLOCK_NUMEL, out) == BLOCK_NUMEL);
  }

cleanup:
  /* cleaning up */
  if(out) fclose(out);
  return err;
}

#endif /* BARREL_H */
