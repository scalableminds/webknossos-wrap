/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import com.google.common.io.{LittleEndianDataInputStream => DataInputStream}
import net.liftweb.common.{Box, Failure, Full}

object BlockType extends Enumeration {
  val Invalid, Raw, LZ4, LZ4HC, Unknown = Value

  def isSupported(blockType: BlockType.Value) = blockType != Invalid && blockType != Unknown

  def isCompressed(blockType: BlockType.Value) = blockType == LZ4 || blockType == LZ4HC
}

object VoxelType extends Enumeration {
  val Invalid, UInt8, UInt16, UInt32, UInt64, Float, Double = Value

  def isSupported(voxelType: VoxelType.Value) = voxelType != Invalid
}

case class WKWHeader(
                          version: Int,
                          numBlocksPerCubeDimension: Int,
                          numVoxelsPerBlockDimension: Int,
                          blockType: BlockType.Value,
                          voxelType: VoxelType.Value,
                          numBytesPerVoxel: Int,
                          dataOffset: Long,
                          jumpTable: Array[Long] = Array.empty
                        ) {
  def numBlocksPerCube: Long = numBlocksPerCubeDimension * numBlocksPerCubeDimension * numBlocksPerCubeDimension

  def numBytesPerBlock: Int = numVoxelsPerBlockDimension * numVoxelsPerBlockDimension * numVoxelsPerBlockDimension * numBytesPerVoxel

  def numBytesPerCube: Long = numBytesPerBlock.toLong * numBlocksPerCube

  def numBytesPerFile: Long = dataOffset + numBytesPerCube

  def isCompressed: Boolean = BlockType.isCompressed(blockType)
}

object WKWHeader {
  val magicBytes = "WKW".getBytes
  val currentVersion = 1

  def apply(dataStream: DataInputStream, skipReadingJumpTable : Boolean): Box[WKWHeader] = {
    // Check magic bytes.
    val magicByteBuffer: Array[Byte] = Array.fill(magicBytes.length) {0}
    dataStream.read(magicByteBuffer, 0, magicBytes.length)
    if (!magicByteBuffer.sameElements(magicBytes)) {
      return Failure("Error reading WKW header: Invalid magic bytes.")
    }

    // Check, if version is supported.
    val version = dataStream.readUnsignedByte()
    if (version != currentVersion) {
      return Failure("Error reading WKW header: Unknown version.")
    }

    val sideLengths = dataStream.readUnsignedByte()
    val numBlocksPerCubeDimension = 1 << (sideLengths >>> 4) // file-side-length [higher nibble]
    val numVoxelsPerBlockDimension = 1 << (sideLengths & 0x0f) // block-side-length [lower nibble]

    // Check, if blockType and voxelType are supported.
    val blockType = BlockType(dataStream.readUnsignedByte())
    if (!BlockType.isSupported(blockType)) {
      return Failure("Error reading WKW header: Specified blockType is not supported.")
    }
    val voxelType = VoxelType(dataStream.readUnsignedByte())
    if (!VoxelType.isSupported(voxelType)) {
      return Failure("Error reading WKW header: Specified voxelType is not supported.")
    }

    val numBytesPerVoxel = dataStream.readUnsignedByte() // voxel-size

    if (skipReadingJumpTable || !BlockType.isCompressed(blockType)) {
      val dataOffset = dataStream.readLong()
      Full(new WKWHeader(version, numBlocksPerCubeDimension, numVoxelsPerBlockDimension, blockType, voxelType, numBytesPerVoxel, dataOffset))
    }

    // Read jump table
    val numBlocksPerCube = numBlocksPerCubeDimension * numBlocksPerCubeDimension * numBlocksPerCubeDimension
    val jumpTable = (0 to numBlocksPerCube).map(_ => dataStream.readLong()).toArray
    Full(new WKWHeader(version, numBlocksPerCubeDimension, numVoxelsPerBlockDimension, blockType, voxelType, numBytesPerVoxel, jumpTable(0), jumpTable))
  }
}
