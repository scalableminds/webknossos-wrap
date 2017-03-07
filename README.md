# The webKNOSSOS Wrapper Format
webKNOSSOS wrapper (wk-wrap) is a file format for large volumetric (i.e., three-
dimensional) voxel data. It was designed for large file sizes and high
performance when reading and writing small subvolumes of data.

Each wk-wrap file contains the data for a cube of voxel, where the cube side-
length is a power of two. Every voxel of this cube contains a fixed number of
values of a given data type (e.g., one 8-bit value for raw image data, or one
32-bit value for image segmentation).

## Implementations

This repository contains the reference implementations for the webKNOSSOS
wrapper format. Code is available for

* C++
* Scala
* MATLAB

The latter is a binding around the C++ library and demonstrates how wk-wrap
files can be read and written from within other programming languages.

## High-level description
Each file contains the data for a cube with side-length (CLEN) of FILE_CLEN
(MUST be a power of two; e.g., 1024) voxels. Within each file, the data is split
into smaller, non-overlapping cubes (called "blocks") with a side-length of
BLOCK_CLEN (MUST be a power of two; e.g., 32) voxels.

To enable fast access to subvolumes of the voxel cube, blocks are stored in
Morton order. That is,
```
  block index           0         1         2         3         4         5
  block coordinates (0, 0, 0) (1, 0, 0) (0, 1, 0) (1, 1, 0) (0, 0, 1) (1, 0, 1)
         6         7         8         9        10        11        12     ...
     (0, 1, 1) (1, 1, 1) (2, 0, 0) (3, 0, 0) (2, 1, 0) (3, 1, 0) (2, 0, 1) ...
```

For further information, see the Wikipedia entry on the [Z-order curve](
https://en.wikipedia.org/wiki/Z-order_curve).

## File format
Each wk-wrap file begins with a file header. Depending on the content of this
header, additional meta data MAY follow. The content of the file header and the
optional meta data MUST be sufficient to determine the offset and size (in
bytes) of each encoded block.

### File header
Each wk-wrap file MUST begin with the following header:

|      | +0x00       | +0x01       | +0x02       | +0x03       |
|------|:-----------:|:-----------:|:-----------:|:-----------:|
| 0x00 | 'W' (0x57)  | 'K' (0x4B)  | 'W' (0x57)  | version     |
| 0x04 | perDimLog2  | blockType   | voxelType   | voxelSize   |
| 0x08 | dataOffset  | dataOffset  | dataOffset  | dataOffset  |
| 0x0C | dataOffset  | dataOffset  | dataOffset  | dataOffset  |

#### Header fields
* __version__ contains the wk-wrap format version as unsigned byte. At the time
  of writing, the only valid version number is 0x01.
* __perDimLog2__ contains two 4-bit values (nibbles). The lower nibble (
  `perDimLog2 & 0x0F`) contains __blocksPerFileDimLog2__, i.e., the log2 of the
  number of blocks per file dimension. The higher nibble (`(perDimLog2 & 0xF0)
  >> 4`) contains __voxelsPerBlockDimLog2__, i.e., the log2 of the number of
  voxels per block dimension. Files and blocks are three-dimensional.
* __blockType__ determines how the individual blocks were encoded. Valid values
  are: 0x01 for RAW encoding, 0x02 for LZ4 compressed, and 0x03 for the high-
  compression version of LZ4.
* __voxelType__ encodes the data type of the voxel values. Valid values are

  | value of voxelType    | 0x01  | 0x02   | 0x03   | 0x04   | 0x05  | 0x06   |
  |-----------------------|-------|--------|--------|--------|-------|--------|
  | data type             | uint8 | uint16 | uint32 | uint64 | float | double |

* __voxelSize__ is an uint8 of the number of bytes per voxel. If the wk-wrap
  file contains a single value per voxel, then voxelSize is equal to the byte
  size of the data type. If the wk-wrap file, however, contains multiple
  channels (e.g., three 8-bit values for RGB), voxelSize is a multiple of the
  data type size.
* __dataOffset__ contains the absolute address of the first byte of the first
  block (relative to the beginning of the file) as unsigned 64-bit integer.

### Byte order
Except when noted otherwise, multi-byte voxel values are stored in little-endian
order. That is, bytes are stored in order of increasing significance.

### Raw blocks
Within raw blocks, the voxel values are stored in Fortran order. That is,
```
  voxel index              X + Y * BLOCK_CLEN + Z * BLOCK_CLEN * BLOCK_CLEN
  voxel coordinates                         (X, Y, Z)
```

In wk-wrap version 0x01, the block with index 0 begins immediately after the
fixed header. The bytes of subsequent blocks are immediately following each
other (i.e., no padding).

### LZ4 compressed blocks
If the file header indicates that the blocks were compressed using LZ4 (by
having a blockType value of either 0x02 or 0x03), the file header is immediately
followed by the jump table.

The jump table is an array of N unsigned 64-bit intergers, where N is the number
of blocks in the file. The n-th entry of the jump table contains the absolute
address (relative to the beginning of the file) of the first byte after the data
of block n.

Note that
* the data of block n begins at address jumpTable[n - 1]
* the data of block n is jumpTable[n] - jumpTable[n - 1] bytes long

The value of jumpTable[-1] is defined as dataOffset. For this reason, it is
convenient to build an extended jump table with the N + 1 unsigned 64-bit
integers starting at the position of the dataOffset field.

Decompression is identical for the standard and the high-compression variants of
LZ4. For a wk-wrap reader, the difference between blockType 0x02 and 0x03 is
only semantic.

Decompression must produce valid raw blocks.
