/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import com.google.common.io.{LittleEndianDataInputStream => DataInputStream}
import java.io.{File, FileInputStream}
import net.liftweb.common.{Box, Failure, Full}

object DataStream {
  def using[A <: { def close(): Unit }, B](param: A)(f: A => B): B = {
    try {
      f(param)
    } finally {
      param.close()
    }
  }

  def apply[T](file: File) = using[DataInputStream, T](new DataInputStream(new FileInputStream(file))) _
}

object BlockType extends Enumeration {
  val Invalid, Raw, LZ4, LZ4HC, Unknown = Value
}

object VoxelType extends Enumeration {
  val Invalid, UInt8, UInt16, UInt32, UInt64, Float, Double = Value
}

case class WKWFileHeader(
                          version: Int,
                          fileLength: Int,
                          blockLength: Int,
                          blockType: BlockType.Value,
                          voxelType: VoxelType.Value,
                          voxelSize: Int,
                          dataOffset: Long
                        ) {
  def blockSize = blockLength * blockLength * blockLength * voxelSize
  def blockCount = fileLength * fileLength * fileLength
  def fileSize = dataOffset + blockSize * blockCount
}

case class WKWFile(file: File, header: WKWFileHeader) {
  protected def mortonEncode(x: Int, y: Int, z: Int): Long = {
    var morton = 0L
    val bitLength = math.ceil(math.log(List(x, y, z).max + 1) / math.log(2)).toInt

    (0 until bitLength).foreach { i =>
      morton |= ((x & (1 << i)) << (2 * i)) |
                ((y & (1 << i)) << (2 * i + 1)) |
                ((z & (1 << i)) << (2 * i + 2))
    }
    morton
  }

  def readBlock(x: Int, y: Int, z: Int): Box[Array[Byte]] = {
    val blockData: Array[Byte] = Array.fill(header.blockSize){0}
    val blockIndex = mortonEncode(x, y, z)

    if (blockIndex >= header.blockCount) {
      return Failure("Failed to read WKW block: Block index out of range.")
    }

    val byteOffset = header.dataOffset + blockIndex * header.blockSize
    DataStream(file) { ds =>
      ds.skip(byteOffset.toInt)
      ds.read(blockData, 0, header.blockSize)
    }
    Full(blockData)
  }
}

object WKWFileHeader {
  val magicBytes = "WKW".getBytes
  val currentVersion = 1

  def apply(file: File): Box[WKWFileHeader] = {
    DataStream(file) { dataStream =>
      // Check magic bytes.
      val magicByteBuffer: Array[Byte] = Array.fill(magicBytes.length){0}
      dataStream.read(magicByteBuffer, 0, magicBytes.length)
      if (!magicByteBuffer.sameElements(magicBytes)) {
        return Failure("Failed reading WKW header: Invalid magic byte sequence.")
      }

      // Check, if version is supported.
      val version = dataStream.readUnsignedByte()
      if (version != currentVersion) {
        return Failure("Failed reading WKW header: Version not supported.")
      }

      val lengths = dataStream.readUnsignedByte()
      val fileLength = math.pow(2, lengths >>> 4).toInt // higher nibble
      val blockLength = math.pow(2, lengths & 0x0f).toInt // lower nibble
      val blockType = BlockType(dataStream.readUnsignedByte())
      val voxelType = VoxelType(dataStream.readUnsignedByte())
      val voxelSize = dataStream.readUnsignedByte()
      val dataOffset = dataStream.readLong()

      Full(new WKWFileHeader(version, fileLength, blockLength, blockType, voxelType, voxelSize, dataOffset))
    }
  }
}

object WKWFile {
  def apply(file: File): Box[WKWFile] = {
    if (!file.exists() || !file.canRead()) {
      return Failure(s"WKWFile: File not found [${file}]")
    }

    for {
      header <- WKWFileHeader(file)
    } yield {
      new WKWFile(file, header)
    }
  }
}
