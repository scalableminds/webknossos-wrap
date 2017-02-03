/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import com.google.common.io.{LittleEndianDataInputStream => DataInputStream}
import java.io.{File, FileInputStream, RandomAccessFile}

import net.liftweb.common.{Box, Empty, Failure, Full}
import resource._

object BlockType extends Enumeration {
  val Invalid, Raw, LZ4, LZ4HC, Unknown = Value
}

object VoxelType extends Enumeration {
  val Invalid, UInt8, UInt16, UInt32, UInt64, Float, Double = Value
}

object ManagedResourceBox {
  def apply[R : Resource, T](resource: R)(f: R => Box[T]): Box[T] = {
    managed(resource).map(f).either.either match {
      case Left(ex) =>
        Failure(s"Exception handling Resource: ${ex.toString}")
      case Right(result) =>
        result
    }
  }
}

case class WKWFileHeader(
                          version: Int,
                          numBlocksPerCubeDimension: Int,
                          numVoxelsPerBlockDimension: Int,
                          blockType: BlockType.Value,
                          voxelType: VoxelType.Value,
                          numBytesPerVoxel: Int,
                          dataOffset: Long
                        ) {
  def numBlocksPerCube: Long = numBlocksPerCubeDimension * numBlocksPerCubeDimension * numBlocksPerCubeDimension
  def numBytesPerBlock: Int = numVoxelsPerBlockDimension * numVoxelsPerBlockDimension * numVoxelsPerBlockDimension * numBytesPerVoxel
  def numBytesPerCube: Long = dataOffset + numBytesPerBlock.toLong * numBlocksPerCube
}

case class WKWFile(file: File, header: WKWFileHeader) {
  protected def mortonEncode(x: Int, y: Int, z: Int): Long = {
    var morton = 0L
    val bitLength = math.ceil(math.log(List(x, y, z).max + 1) / math.log(2)).toInt

    (0 until bitLength).foreach { i =>
      morton |= ((x & (1L << i)) << (2 * i)) |
        ((y & (1L << i)) << (2 * i + 1)) |
        ((z & (1L << i)) << (2 * i + 2))
    }
    morton
  }

  def readBlock(x: Int, y: Int, z: Int): Box[Array[Byte]] = {
    if (x < 0 || x >= header.numBlocksPerCubeDimension ||
        y < 0 || y >= header.numBlocksPerCubeDimension ||
        z < 0 || z >= header.numBlocksPerCubeDimension) {
      return Failure("Failed to read WKW block: Block coordinates out of range.")
    }

    val blockData: Array[Byte] = Array.fill(header.numBytesPerBlock){0}
    val blockIndex = mortonEncode(x, y, z)

    if (blockIndex >= header.numBlocksPerCube) {
      return Failure("Failed to read WKW block: Block index out of range.")
    }

    val byteOffset = header.dataOffset + blockIndex * header.numBytesPerBlock
    ManagedResourceBox(new RandomAccessFile(file, "r")) { wkwFile =>
      wkwFile.seek(byteOffset)
      wkwFile.read(blockData, 0, header.numBytesPerBlock)
      Full(blockData)
    }
  }
}

object WKWFileHeader {
  val magicBytes = "WKW".getBytes
  val currentVersion = 1

  def apply(file: File): Box[WKWFileHeader] = {
    ManagedResourceBox[DataInputStream, WKWFileHeader](new DataInputStream(new FileInputStream(file))) { dataStream =>
      // Check magic bytes.
      val magicByteBuffer: Array[Byte] = Array.fill(magicBytes.length) {
        0
      }
      dataStream.read(magicByteBuffer, 0, magicBytes.length)
      if (!magicByteBuffer.sameElements(magicBytes)) {
        return Failure("Failed reading WKW header: Invalid magic byte sequence.")
      }

      // Check, if version is supported.
      val version = dataStream.readUnsignedByte()
      if (version != currentVersion) {
        return Failure("Failed reading WKW header: Version not supported.")
      }

      val sideLengths = dataStream.readUnsignedByte()
      val numBlocksPerCubeDimension = 1 << (sideLengths >>> 4) // file-side-length [higher nibble]
      val numVoxelsPerBlockDimension = 1 << (sideLengths & 0x0f) // block-side-length [lower nibble]
      val blockType = BlockType(dataStream.readUnsignedByte())
      val voxelType = VoxelType(dataStream.readUnsignedByte())
      val numBytesPerVoxel = dataStream.readUnsignedByte() // voxel-size
      val dataOffset = dataStream.readLong()
      Full(new WKWFileHeader(version, numBlocksPerCubeDimension, numVoxelsPerBlockDimension, blockType, voxelType, numBytesPerVoxel, dataOffset))
    }
  }
}

object WKWFile {
  def apply(file: File): Box[WKWFile] = {
    if (!file.exists() || !file.canRead()) {
      Failure(s"WKWFile: File not found [${file}]")
    } else {
      for {
        header <- WKWFileHeader(file)
      } yield {
        new WKWFile(file, header)
      }
    }
  }
}
