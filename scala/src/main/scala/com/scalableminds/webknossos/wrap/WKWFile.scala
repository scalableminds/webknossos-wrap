/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import com.google.common.io.{LittleEndianDataInputStream => DataInputStream}
import com.scalableminds.webknossos.wrap.util.ResourceBox
import com.scalableminds.webknossos.wrap.util.BoxHelpers._
import com.scalableminds.webknossos.wrap.util.ExtendedTypes.ExtendedRandomAccessFile
import java.io.{File, FileInputStream, RandomAccessFile}
import net.jpountz.lz4.LZ4Factory
import net.liftweb.common.{Box, Failure, Full}

object FileMode extends Enumeration {
  val Read, ReadWrite = Value
}


case class WKWFile(header: WKWHeader, fileMode: FileMode.Value, underlyingFile: RandomAccessFile) {
  lazy val lz4Decompressor = LZ4Factory.fastestInstance().fastDecompressor()

  private def mortonEncode(x: Int, y: Int, z: Int): Long = {
    var morton = 0L
    val bitLength = math.ceil(math.log(List(x, y, z).max + 1) / math.log(2)).toInt

    (0 until bitLength).foreach { i =>
      morton |= ((x & (1L << i)) << (2 * i)) |
        ((y & (1L << i)) << (2 * i + 1)) |
        ((z & (1L << i)) << (2 * i + 2))
    }
    morton
  }

  private def readCompressedBlock(mortonIndex: Long): Box[Array[Byte]] = {
    val blockOffset = header.jumpTable(header.numBytesPerBlock)
    val compressedLength = (header.jumpTable(header.numBytesPerBlock + 1) - blockOffset).toInt
    val compressedData: Array[Byte] = Array.fill(compressedLength) {0}
    val blockData: Array[Byte] = Array.fill(compressedLength) {0}

    underlyingFile.seek(blockOffset)
    underlyingFile.read(compressedData, 0, compressedLength)

    for {
      decompressedLength <- Try(lz4Decompressor.decompress(compressedData, blockData, header.numBytesPerBlock))
      _ <- Check(decompressedLength == header.numBytesPerBlock) ~> "Error reading WKW block: Decompressed block has invalid length."
    } yield {
      blockData
    }
  }

  private def readUncompressedBlock(mortonIndex: Long): Box[Array[Byte]] = {
    val blockOffset = header.dataOffset + mortonIndex * header.numBytesPerBlock
    val blockData: Array[Byte] = Array.fill(header.numBytesPerBlock) {0}

    underlyingFile.seek(blockOffset)
    underlyingFile.read(blockData, 0, header.numBytesPerBlock)
    Full(blockData)
  }

  private def readBlock(mortonIndex: Long): Box[Array[Byte]] = {
    if (mortonIndex < 0 || mortonIndex >= header.numBlocksPerCube) {
      return Failure("Error reading WKW block: Morton index is out of range.")
    }

    if (underlyingFile.isClosed) {
      return Failure("Error reading WKW block: underlying file has bee closed.")
    }

    if (header.isCompressed) {
      readCompressedBlock(mortonIndex)
    } else {
      readUncompressedBlock(mortonIndex)
    }
  }

  def readBlock(x: Int, y: Int, z: Int): Box[Array[Byte]] = {
    if (x < 0 || x >= header.numBlocksPerCubeDimension ||
        y < 0 || y >= header.numBlocksPerCubeDimension ||
        z < 0 || z >= header.numBlocksPerCubeDimension) {
      return Failure("Error reading WKW block: Block coordinates are out of range.")
    }

    val mortonIndex = mortonEncode(x, y, z)
    readBlock(mortonIndex)
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
  private def fileModeString(isCompressed: Boolean, fileMode: FileMode.Value): Box[String] = {
    fileMode match {
      case FileMode.Read =>
        Full("r")
      case FileMode.ReadWrite =>
        if (isCompressed) {
          Failure("Error reading WKW file: Compressed files can only be opened in read-only.")
        } else {
          Full("rw")
        }
    }
  }

  def apply(file: File, fileMode: FileMode.Value = FileMode.Read): Box[WKWFile] = {
    for {
      header <- readHeader(file, false)
      mode <- fileModeString(header.isCompressed, fileMode)
      underlyingFile <- ResourceBox(new RandomAccessFile(file, mode))
    } yield {
      new WKWFile(header, fileMode, underlyingFile)
    }
  }

  def readHeader(file: File, skipReadingJumpTable: Boolean = true): Box[WKWHeader] = {
    ResourceBox.manage(new DataInputStream(new FileInputStream(file))) { dataStream =>
      for {
        header <- WKWHeader(dataStream, skipReadingJumpTable)
        _ <- Check(header.numBytesPerFile == file.length()) ~> "Error reading WKW header: Unexpected file size."
      } yield {
        header
      }
    }
  }
}
