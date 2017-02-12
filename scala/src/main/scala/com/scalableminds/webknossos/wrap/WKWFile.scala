/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import com.scalableminds.webknossos.wrap.util.ResourceBox
import com.scalableminds.webknossos.wrap.util.BoxHelpers._
import com.scalableminds.webknossos.wrap.util.ExtendedTypes.ExtendedRandomAccessFile
import java.io.{File, RandomAccessFile}
import net.jpountz.lz4.LZ4Factory
import net.liftweb.common.{Box, Failure, Full}

object FileMode extends Enumeration {
  val Read, ReadWrite = Value
}

case class WKWFile(header: WKWHeader, fileMode: FileMode.Value, underlyingFile: RandomAccessFile) {
  private lazy val lz4Decompressor = LZ4Factory.fastestInstance().fastDecompressor()

  private def error(msg: String, expected: Any, actual: Any) = {
    WKWFile.error(msg, expected, actual, new File(underlyingFile.getPath))
  }

  private def mortonEncode(x: Int, y: Int, z: Int): Int = {
    var morton = 0
    val bitLength = math.ceil(math.log(List(x, y, z).max + 1) / math.log(2)).toInt

    (0 until bitLength).foreach { i =>
      morton |= ((x & (1 << i)) << (2 * i)) |
        ((y & (1 << i)) << (2 * i + 1)) |
        ((z & (1 << i)) << (2 * i + 2))
    }
    morton
  }

  private def readUncompressedBlock(mortonIndex: Int): Box[Array[Byte]] = {
    val blockOffset = header.dataOffset + mortonIndex.toLong * header.numBytesPerBlock.toLong
    val blockData: Array[Byte] = Array.fill(header.numBytesPerBlock) {0}

    underlyingFile.seek(blockOffset)
    underlyingFile.read(blockData, 0, header.numBytesPerBlock)
    Full(blockData)
  }

  private def readCompressedBlock(mortonIndex: Int): Box[Array[Byte]] = {
    val blockOffset = header.jumpTable(mortonIndex)
    val compressedLength = (header.jumpTable(mortonIndex + 1) - blockOffset).toInt
    val compressedData: Array[Byte] = Array.fill(compressedLength) {0}
    val uncompressedData: Array[Byte] = Array.fill(header.numBytesPerBlock) {0}

    underlyingFile.seek(blockOffset)
    underlyingFile.read(compressedData, 0, compressedLength)

    for {
      bytesDecompressed <- Try(lz4Decompressor.decompress(compressedData, uncompressedData, header.numBytesPerBlock))
      _ <- Check(bytesDecompressed == compressedLength) ?~! error("Decompressed unexpected number of bytes", compressedLength, bytesDecompressed)
    } yield {
      uncompressedData
    }
  }

  private def readBlock(mortonIndex: Int): Box[Array[Byte]] = {
    for {
      data <- if (header.isCompressed) readCompressedBlock(mortonIndex) else readUncompressedBlock(mortonIndex)
    } yield {
      data
    }
  }

  def readBlock(x: Int, y: Int, z: Int): Box[Array[Byte]] = {
    for {
      _ <- Check(x >= 0 && x < header.numBlocksPerCubeDimension) ?~! error("X coordinate is out of range", s"[0, ${header.numBlocksPerCubeDimension})", x)
      _ <- Check(y >= 0 && y < header.numBlocksPerCubeDimension) ?~! error("Y coordinate is out of range", s"[0, ${header.numBlocksPerCubeDimension})", y)
      _ <- Check(z >= 0 && z < header.numBlocksPerCubeDimension) ?~! error("Z coordinate is out of range", s"[0, ${header.numBlocksPerCubeDimension})", z)
      mortonIndex = mortonEncode(x, y, z)
      _ <- Check(mortonIndex >= 0 && mortonIndex < header.numBlocksPerCube) ?~! error("Morton index is out of range", s"[0, ${header.numBlocksPerCube}", mortonIndex)
      data <- readBlock(mortonIndex)
    } yield {
      data
    }
  }

  def close() {
    if (!underlyingFile.isClosed) {
      underlyingFile.close()
    }
  }

  def compress(file: File, target: Option[File] = None): Box[Unit] = {
    Failure("Not implemented yet!")
  }

  def decompress(file: File, target: Option[File] = None): Box[Unit] = {
    Failure("Not implemented yet!")
  }
}

object WKWFile {
  private def error(msg: String, file: File): String = {
    s"""Error processing WKW file: ${msg} [file: ${file.getPath}]."""
  }

  private def error(msg: String, expected: Any, actual: Any, file: File): String = {
    s"""Error processing WKW file: ${msg} [expected: ${expected}, actual: ${actual}, file: ${file.getPath}]."""
  }

  private def fileModeString(file: File, isCompressed: Boolean, fileMode: FileMode.Value): Box[String] = {
    fileMode match {
      case FileMode.Read =>
        Full("r")
      case FileMode.ReadWrite =>
        if (isCompressed) {
          Failure(error("Compressed files can only be opened read-only", file))
        } else {
          Full("rw")
        }
    }
  }

  def apply(file: File, fileMode: FileMode.Value = FileMode.Read): Box[WKWFile] = {
    for {
      header <- WKWHeader(file, true)
      _ <- Check(header.expectedFileSize == file.length) ?~! error("Unexpected file size", header.expectedFileSize, file.length, file)
      mode <- fileModeString(file, header.isCompressed, fileMode)
      underlyingFile <- ResourceBox(new RandomAccessFile(file, mode))
    } yield {
      new WKWFile(header, fileMode, underlyingFile)
    }
  }
}
