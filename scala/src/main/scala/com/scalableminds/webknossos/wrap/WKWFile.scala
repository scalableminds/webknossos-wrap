/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import com.newrelic.api.agent.NewRelic
import com.scalableminds.webknossos.wrap.util.ResourceBox
import com.scalableminds.webknossos.wrap.util.BoxHelpers._
import com.scalableminds.webknossos.wrap.util.ExtendedTypes._
import java.io.{File, RandomAccessFile}
import java.nio.MappedByteBuffer
import java.nio.channels.FileChannel
import java.nio.file.{Files, Paths, StandardCopyOption}

import net.jpountz.lz4.LZ4Factory
import net.liftweb.common.{Box, Failure, Full}
import org.xerial.snappy.Snappy

object FileMode extends Enumeration {
  val Read, ReadWrite = Value
}

case class WKWFile(header: WKWHeader, fileMode: FileMode.Value, underlyingFile: RandomAccessFile) {
  private lazy val lz4Decompressor = LZ4Factory.nativeInstance().fastDecompressor()
  private lazy val lz4FastCompressor = LZ4Factory.nativeInstance().fastCompressor()
  private lazy val lz4HighCompressor = LZ4Factory.nativeInstance().highCompressor()

  val mappedBuffers: Array[MappedByteBuffer] = mapBuffers

  private def mapBuffers: Array[MappedByteBuffer] = {
    val channel = underlyingFile.getChannel()
    val mapMode = fileMode match {
      case FileMode.Read =>
        FileChannel.MapMode.READ_ONLY
      case FileMode.ReadWrite =>
        FileChannel.MapMode.READ_WRITE
    }
    (0L to underlyingFile.length by Int.MaxValue.toLong).map { offset =>
      val length = Math.min(Int.MaxValue, underlyingFile.length - offset)
      channel.map(mapMode, offset, length)
    }.toArray
  }

  private def error(msg: String) = {
    WKWFile.error(msg, new File(underlyingFile.getPath))
  }

  private def error(msg: String, expected: Any, actual: Any) = {
    WKWFile.error(msg, expected, actual, new File(underlyingFile.getPath))
  }

  def readFromUnderlyingBuffers(offset: Long, length: Int): Array[Byte] = {
    val dest = Array.ofDim[Byte](length)
    val bufferIndex = (offset / Int.MaxValue).toInt
    val bufferOffset = (offset % Int.MaxValue).toInt
    val buffer = mappedBuffers(bufferIndex)

    if (buffer.capacity - bufferOffset < length) {
      val firstPart: Int = buffer.capacity - bufferOffset
      val secondPart = length - firstPart
      buffer.copyTo(bufferOffset, dest, 0, firstPart)
      mappedBuffers(bufferIndex + 1).copyTo(0, dest, firstPart, secondPart)
    } else {
      buffer.copyTo(bufferOffset, dest, 0, length)
    }
    dest
  }

  private def writeToBuffers(offset: Long, data: Array[Byte]): Unit = {
    val bufferIndex = (offset / Int.MaxValue).toInt
    val bufferOffset = (offset % Int.MaxValue).toInt
    Array.copy(data, 0, mappedBuffers(bufferIndex).array(), bufferOffset, data.length)
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

  private def computeMortonIndex(x: Int, y: Int, z: Int): Box[Int] = {
    for {
      _ <- Check(x >= 0 && x < header.numBlocksPerCubeDimension) ?~! error("X coordinate is out of range", s"[0, ${header.numBlocksPerCubeDimension})", x)
      _ <- Check(y >= 0 && y < header.numBlocksPerCubeDimension) ?~! error("Y coordinate is out of range", s"[0, ${header.numBlocksPerCubeDimension})", y)
      _ <- Check(z >= 0 && z < header.numBlocksPerCubeDimension) ?~! error("Z coordinate is out of range", s"[0, ${header.numBlocksPerCubeDimension})", z)
    } yield {
      mortonEncode(x, y, z)
    }
  }

  private def compressBlock(targetBlockType: BlockType.Value = header.blockType)(rawBlock: Array[Byte]): Box[Array[Byte]] = {
    val t = System.currentTimeMillis
    val result = targetBlockType match {
      case BlockType.LZ4 | BlockType.LZ4HC =>
        val compressor = if (targetBlockType == BlockType.LZ4) lz4FastCompressor else lz4HighCompressor
        val maxCompressedLength = compressor.maxCompressedLength(rawBlock.length)
        val compressedBlock = Array.ofDim[Byte](maxCompressedLength)
        Try(compressor.compress(rawBlock, compressedBlock)).map { compressedLength =>
          compressedBlock.slice(0, compressedLength)
        }
      case BlockType.Snappy =>
        Try(Snappy.compress(rawBlock))
      case BlockType.Raw =>
        Full(rawBlock)
      case _ =>
        Failure(error("Invalid targetBlockType for compression"))
    }
    NewRelic.recordResponseTimeMetric(s"Custom/WebknossosWrap/block-compress-time-${header.blockType}", System.currentTimeMillis - t)
    result
  }

  private def decompressBlock(sourceBlockType: BlockType.Value = header.blockType)(compressedBlock: Array[Byte]): Box[Array[Byte]] = {
    val t = System.currentTimeMillis

    val result = sourceBlockType match {
      case BlockType.LZ4 | BlockType.LZ4HC =>
        val rawBlock: Array[Byte] = Array.ofDim[Byte](header.numBytesPerBlock)
        for {
          bytesDecompressed <- Try(lz4Decompressor.decompress(compressedBlock, rawBlock, header.numBytesPerBlock))
          _ <- Check(bytesDecompressed == compressedBlock.length) ?~! error("Decompressed unexpected number of bytes", compressedBlock.length, bytesDecompressed)
        } yield {
          rawBlock
        }
      case BlockType.Snappy =>
        Try(Snappy.uncompress(compressedBlock))
      case BlockType.Raw =>
        Full(compressedBlock)
      case _ =>
        Failure(error("Invalid sourceBlockType for decompression"))
    }
    NewRelic.recordResponseTimeMetric(s"Custom/WebknossosWrap/block-decompress-time-${header.blockType}", System.currentTimeMillis - t)
    result
  }

  private def readUncompressedBlock(mortonIndex: Int): Array[Byte] = {
    val blockOffset = header.dataOffset + mortonIndex.toLong * header.numBytesPerBlock.toLong
    readFromUnderlyingBuffers(blockOffset, header.numBytesPerBlock)
  }

  private def writeUncompressedBlock(mortonIndex: Int, blockData: Array[Byte]) = {
    val blockOffset = header.dataOffset + mortonIndex.toLong * header.numBytesPerBlock.toLong
    writeToBuffers(blockOffset, blockData)
  }

  private def readCompressedBlock(mortonIndex: Int): Box[Array[Byte]] = {
    val blockOffset = header.jumpTable(mortonIndex)
    val compressedLength = (header.jumpTable(mortonIndex + 1) - blockOffset).toInt
    val compressedBlock = readFromUnderlyingBuffers(blockOffset, compressedLength)
    decompressBlock()(compressedBlock)
  }

  def readBlock(x: Int, y: Int, z: Int): Box[Array[Byte]] = {
    val t = System.currentTimeMillis
    for {
      _ <- Check(!underlyingFile.isClosed) ?~! error("File is already closed")
      mortonIndex <- computeMortonIndex(x, y, z)
      data <- if (header.isCompressed) Try(readCompressedBlock(mortonIndex)) else Try(readUncompressedBlock(mortonIndex))
    } yield {
      NewRelic.recordResponseTimeMetric(s"Custom/WebknossosWrap/block-read-time-${header.blockType}", System.currentTimeMillis - t)
      data
    }
  }

  def writeBlock(x: Int, y: Int, z: Int, data: Array[Byte]): Box[Unit] = {
    val t = System.currentTimeMillis
    for {
      _ <- Check(!underlyingFile.isClosed) ?~! error("File is already closed")
      _ <- Check(fileMode == FileMode.ReadWrite) ?~! error("Cannot write to files opened read-only")
      _ <- Check(!header.isCompressed) ?~! error("Cannot write to compressed files")
      _ <- Check(data.length == header.numBytesPerBlock) ?~! error("Data to be written has invalid length", header.numBytesPerBlock, data.length)
      mortonIndex <- computeMortonIndex(x, y, z)
      _ <- Try(writeUncompressedBlock(mortonIndex, data))
    } yield {
      NewRelic.recordResponseTimeMetric(s"Custom/WebknossosWrap/block-write-time-${header.blockType}", System.currentTimeMillis - t)
    }
  }

  def close() {
    if (!underlyingFile.isClosed) {
      underlyingFile.close()
    }
  }

  private def moveFile(tempFile: File, targetFile: File) = {
    Files.move(tempFile.toPath, Paths.get(underlyingFile.getPath), StandardCopyOption.REPLACE_EXISTING)
    close()
  }

  private def changeBlockType(targetBlockType: BlockType.Value): Box[WKWFile] = {
    val tempFile = new File(underlyingFile.getPath + ".tmp")
    val targetFile = new File(underlyingFile.getPath)
    val toCompressed = BlockType.isCompressed(targetBlockType)
    val jumpTableSize = if (toCompressed) header.numBlocksPerCube + 1 else 1
    val tempHeader = header.copy(blockType = targetBlockType, jumpTable = Array.ofDim[Long](jumpTableSize))

    for {
      _ <- Check(BlockType.isSupported(targetBlockType)) ?~! error("TargetBlockType is not supported")
      _ <- Check(targetBlockType != header.blockType) ?~! error("File already has requested blockType")
      _ <- ResourceBox.manage(new RandomAccessFile(tempFile, "rw")) { file =>
        tempHeader.writeToFile(file)
        val dataOffset = file.getFilePointer

        underlyingFile.seek(header.dataOffset)
        val sourceBlockLengths = if (header.isCompressed) {
          header.jumpTable.sliding(2).map(a => (a(1) - a(0)).toInt)
        } else {
          Array.fill(header.numBlocksPerCube){header.numBytesPerBlock}.toIterator
        }
        val targetBlockLengths = sourceBlockLengths.foldLeft[Box[Seq[Int]]](Full(Seq.empty)) {
          case (Full(result), blockLength) =>
            val blockData = Array.ofDim[Byte](blockLength)
            underlyingFile.read(blockData)
            for {
              rawBlock <- decompressBlock(header.blockType)(blockData)
              encodedBlock <- compressBlock(targetBlockType)(rawBlock)
            } yield {
              file.write(encodedBlock)
              result :+ encodedBlock.length
            }
          case (failure, _) =>
            failure
        }

        targetBlockLengths.map { blockLengths =>
          val jumpTable = if (toCompressed) {
            blockLengths.map(_.toLong).scan(dataOffset)(_ + _).toArray
          } else {
            Array(dataOffset)
          }
          val newHeader = tempHeader.copy(jumpTable = jumpTable)
          file.seek(0)
          newHeader.writeToFile(file)
        }
      }
      _ <- Try(moveFile(tempFile, targetFile))
      wkwFile <- WKWFile(targetFile, fileMode)
    } yield {
      wkwFile
    }
  }

  def decompress: Box[WKWFile] = changeBlockType(BlockType.Raw)

  def compress(targetBlockType: BlockType.Value): Box[WKWFile] = changeBlockType(targetBlockType)
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
