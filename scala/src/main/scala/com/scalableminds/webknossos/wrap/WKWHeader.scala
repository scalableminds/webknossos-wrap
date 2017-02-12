/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import com.google.common.io.{LittleEndianDataInputStream => DataInputStream}
import com.scalableminds.webknossos.wrap.util.ResourceBox
import com.scalableminds.webknossos.wrap.util.BoxHelpers._
import java.io.{File, FileInputStream}
import net.liftweb.common.{Box, Full}

object BlockType extends Enumeration {
  val Invalid, Raw, LZ4, LZ4HC, Unknown = Value

  def isSupported(blockType: BlockType.Value) = blockType != Invalid && blockType != Unknown

  def isCompressed(blockType: BlockType.Value) = blockType == LZ4 || blockType == LZ4HC

  def isUncompressed(blockType: BlockType.Value) = blockType == Raw
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
                          jumpTable: Array[Long]
                        ) {
  def isCompressed: Boolean = BlockType.isCompressed(blockType)

  def numBlocksPerCube: Int = numBlocksPerCubeDimension * numBlocksPerCubeDimension * numBlocksPerCubeDimension

  def numBytesPerBlock: Int = numVoxelsPerBlockDimension * numVoxelsPerBlockDimension * numVoxelsPerBlockDimension * numBytesPerVoxel

  def expectedFileSize: Long = {
    if (isCompressed) {
      jumpTable.lastOption.getOrElse(-1L)
    } else {
      dataOffset + numBytesPerBlock.toLong * numBlocksPerCube.toLong
    }
  }
}

object WKWHeader {
  private def error(msg: String, expected: Any, actual: Any, file: File): String = {
    s"""Error reading WKW header: ${msg} [expected: ${expected}, actual: ${actual}, file: ${file.getPath}]."""
  }

  private def error(msg: String, file: File): String = {
    s"""Error reading WKW header: ${msg} [file: ${file.getPath}]."""
  }

  val magicBytes = "WKW".getBytes
  val currentVersion = 1

  def apply(file: File, readJumpTable: Boolean = false): Box[WKWHeader] = {
    ResourceBox.manage(new DataInputStream(new FileInputStream(file))) { dataStream =>
      val magicByteBuffer: Array[Byte] = Array.fill(magicBytes.length) {0}
      dataStream.read(magicByteBuffer, 0, magicBytes.length)
      val version = dataStream.readUnsignedByte()
      val sideLengths = dataStream.readUnsignedByte()
      val numBlocksPerCubeDimension = 1 << (sideLengths >>> 4) // fileSideLength [higher nibble]
      val numVoxelsPerBlockDimension = 1 << (sideLengths & 0x0f) // blockSideLength [lower nibble]
      val blockType = BlockType(dataStream.readUnsignedByte())
      val voxelType = VoxelType(dataStream.readUnsignedByte())
      val numBytesPerVoxel = dataStream.readUnsignedByte() // voxel-size

      for {
        _ <- Check(magicByteBuffer.sameElements(magicBytes)) ?~! error("Invalid magic bytes", magicBytes, magicByteBuffer, file)
        _ <- Check(version == currentVersion) ?~! error("Unknown version", currentVersion, version, file)
        // We only support fileSideLengths < 1024, so that the total number of blocks per file fits in an Int.
        _ <- Check(numBlocksPerCubeDimension < 1024) ?~! error("Specified fileSideLength not supported", numBlocksPerCubeDimension, "[0, 1024)", file)
        // We only support blockSideLengths < 1024, so that the total number of voxels per block fits in an Int.
        _ <- Check(numBlocksPerCubeDimension < 1024) ?~! error("Specified blockSideLength not supported", numVoxelsPerBlockDimension, "[0, 1024)", file)
        _ <- Check(BlockType.isSupported(blockType)) ?~! error("Specified blockType is not supported", file)
        _ <- Check(VoxelType.isSupported(voxelType)) ?~! error("Specified voxelType is not supported", file)
      } yield {
        if (BlockType.isCompressed(blockType) && readJumpTable) {
          // Read jump table
          val numBlocksPerCube = numBlocksPerCubeDimension * numBlocksPerCubeDimension * numBlocksPerCubeDimension
          val jumpTable = (0 to numBlocksPerCube).map(_ => dataStream.readLong()).toArray
          new WKWHeader(version, numBlocksPerCubeDimension, numVoxelsPerBlockDimension, blockType, voxelType, numBytesPerVoxel, jumpTable(0), jumpTable)
        } else {
          val dataOffset = dataStream.readLong()
          new WKWHeader(version, numBlocksPerCubeDimension, numVoxelsPerBlockDimension, blockType, voxelType, numBytesPerVoxel, dataOffset, Array.empty)
        }
      }
    }
  }
}
