/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import java.io._
import java.nio.channels.FileChannel

import com.google.common.io.{LittleEndianDataInputStream => DataInputStream}
import com.newrelic.api.agent.NewRelic
import com.scalableminds.webknossos.wrap.util.BoxHelpers._
import com.scalableminds.webknossos.wrap.util.ExtendedTypes._
import com.scalableminds.webknossos.wrap.util.ResourceBox
import net.jpountz.lz4.LZ4Factory
import net.liftweb.common.{Box, Failure, Full}

object FileMode extends Enumeration {
  val Read, ReadWrite = Value
}

trait WKWMortonHelper {

  protected def mortonEncode(x: Int, y: Int, z: Int): Int = {
    var morton = 0
    val bitLength = math.ceil(math.log(List(x, y, z).max + 1) / math.log(2)).toInt

    (0 until bitLength).foreach { i =>
      morton |= ((x & (1 << i)) << (2 * i)) |
        ((y & (1 << i)) << (2 * i + 1)) |
        ((z & (1 << i)) << (2 * i + 2))
    }
    morton
  }

  protected def mortonDecode(mortonIndex: Long): (Int, Int, Int) = {
    var morton = mortonIndex
    var x = 0
    var y = 0
    var z = 0
    var bit = 0

    while (morton > 0) {
      x |= ((morton & 1) << bit).toInt
      morton >>= 1
      y |= ((morton & 1) << bit).toInt
      morton >>= 1
      z |= ((morton & 1) << bit).toInt
      morton >>= 1
      bit += 1
    }
    (x, y, z)
  }
}

trait WKWCompressionHelper {

  protected def error(msg: String): String =
    s"""Error processing WKW file: ${msg}."""

  protected def error(msg: String, expected: Any, actual: Any): String =
    s"""Error processing WKW file: ${msg} [expected: ${expected}, actual: ${actual}]."""

  private lazy val lz4Decompressor = LZ4Factory.nativeInstance().fastDecompressor()

  private lazy val lz4FastCompressor = LZ4Factory.nativeInstance().fastCompressor()

  private lazy val lz4HighCompressor = LZ4Factory.nativeInstance().highCompressor()

  protected def compressBlock(targetBlockType: BlockType.Value)(rawBlock: Array[Byte]): Box[Array[Byte]] = {
    val t = System.currentTimeMillis
    val result = targetBlockType match {
      case BlockType.LZ4 | BlockType.LZ4HC =>
        val compressor = if (targetBlockType == BlockType.LZ4) lz4FastCompressor else lz4HighCompressor
        val maxCompressedLength = compressor.maxCompressedLength(rawBlock.length)
        val compressedBlock = Array.ofDim[Byte](maxCompressedLength)
        Try(compressor.compress(rawBlock, compressedBlock)).map { compressedLength =>
          compressedBlock.slice(0, compressedLength)
        }
      case BlockType.Raw =>
        Full(rawBlock)
      case _ =>
        Failure(error("Invalid targetBlockType for compression"))
    }
    NewRelic.recordResponseTimeMetric(s"Custom/WebknossosWrap/block-compress-time-${targetBlockType}", System.currentTimeMillis - t)
    result
  }

  protected def decompressBlock(sourceBlockType: BlockType.Value, numBytesPerBlock: Int)(compressedBlock: Array[Byte]): Box[Array[Byte]] = {
    val t = System.currentTimeMillis

    val result = sourceBlockType match {
      case BlockType.LZ4 | BlockType.LZ4HC =>
        val rawBlock: Array[Byte] = Array.ofDim[Byte](numBytesPerBlock)
        for {
          bytesDecompressed <- Try(lz4Decompressor.decompress(compressedBlock, rawBlock, numBytesPerBlock))
          _ <- Check(bytesDecompressed == compressedBlock.length) ?~! error("Decompressed unexpected number of bytes", compressedBlock.length, bytesDecompressed)
        } yield {
          rawBlock
        }
      case BlockType.Raw =>
        Full(compressedBlock)
      case _ =>
        Failure(error("Invalid sourceBlockType for decompression"))
    }
    NewRelic.recordResponseTimeMetric(s"Custom/WebknossosWrap/block-decompress-time-${sourceBlockType}", System.currentTimeMillis - t)
    result
  }
}

class WKWFile(val header: WKWHeader, fileMode: FileMode.Value, underlyingFile: RandomAccessFile) extends WKWCompressionHelper with WKWMortonHelper {

  private val channel = underlyingFile.getChannel()

  private val mappedBuffers: Array[ExtendedMappedByteBuffer] = mapBuffers

  private def mapBuffers: Array[ExtendedMappedByteBuffer] = {
    val mapMode = fileMode match {
      case FileMode.Read =>
        FileChannel.MapMode.READ_ONLY
      case FileMode.ReadWrite =>
        FileChannel.MapMode.READ_WRITE
    }
    (0L until underlyingFile.length by Int.MaxValue.toLong).toArray.map { offset =>
      val length = Math.min(Int.MaxValue, underlyingFile.length - offset)
      new ExtendedMappedByteBuffer(channel.map(mapMode, offset, length))
    }
  }

  private def readFromUnderlyingBuffers(offset: Long, length: Int): Array[Byte] = {
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

  private def writeToUnderlyingBuffers(offset: Long, data: Array[Byte]): Unit = {
    val bufferIndex = (offset / Int.MaxValue).toInt
    val bufferOffset = (offset % Int.MaxValue).toInt
    val buffer = mappedBuffers(bufferIndex)

    if (buffer.capacity - bufferOffset < data.length) {
      val firstPart: Int = buffer.capacity - bufferOffset
      val secondPart = data.length - firstPart
      buffer.copyFrom(bufferOffset, data, 0, firstPart)
      mappedBuffers(bufferIndex + 1).copyFrom(0, data, firstPart, secondPart)
    } else {
      buffer.copyFrom(bufferOffset, data, 0, data.length)
    }
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

  def readBlock(x: Int, y: Int, z: Int): Box[Array[Byte]] = {
    val t = System.currentTimeMillis
    for {
      _ <- Check(!underlyingFile.isClosed) ?~! error("File is already closed")
      mortonIndex <- computeMortonIndex(x, y, z)
      (offset, length) <- header.blockBoundaries(mortonIndex)
      data <- Try(readFromUnderlyingBuffers(offset, length))
      decompressedData <- if (header.isCompressed) Try(decompressBlock(header.blockType, header.numBytesPerBlock)(data)) else Try(data)
    } yield {
      NewRelic.recordResponseTimeMetric(s"Custom/WebknossosWrap/block-read-time-${header.blockType}", System.currentTimeMillis - t)
      decompressedData
    }
  }

  def writeBlock(x: Int, y: Int, z: Int, data: Array[Byte]): Box[Unit] = {
    val t = System.currentTimeMillis
    for {
      _ <- Check(!underlyingFile.isClosed) ?~! error("File is already closed")
      _ <- Check(fileMode == FileMode.ReadWrite) ?~! error("Cannot write to erad-only files")
      _ <- Check(!header.isCompressed) ?~! error("Cannot write to compressed files")
      _ <- Check(data.length == header.numBytesPerBlock) ?~! error("Data to be written has invalid length", header.numBytesPerBlock, data.length)
      mortonIndex <- computeMortonIndex(x, y, z)
      (offset, _) <- header.blockBoundaries(mortonIndex)
      _ <- Try(writeToUnderlyingBuffers(offset, data))
    } yield {
      NewRelic.recordResponseTimeMetric(s"Custom/WebknossosWrap/block-write-time-${header.blockType}", System.currentTimeMillis - t)
    }
  }

  def close(): Unit = {
    channel.close()
    underlyingFile.close()
  }
}

object WKWFile extends WKWCompressionHelper {

  private def fileModeString(file: File, isCompressed: Boolean, fileMode: FileMode.Value): Box[String] = {
    fileMode match {
      case FileMode.Read =>
        Full("r")
      case FileMode.ReadWrite =>
        if (isCompressed) {
          Failure(error("Compressed files can only be opened read-only"))
        } else {
          Full("rw")
        }
    }
  }

  def apply(file: File, fileMode: FileMode.Value = FileMode.Read): Box[WKWFile] = {
    for {
      header <- WKWHeader(file, true)
      _ <- Check(header.expectedFileSize == file.length) ?~! error("Unexpected file size", header.expectedFileSize, file.length)
      mode <- fileModeString(file, header.isCompressed, fileMode)
      underlyingFile <- ResourceBox(new RandomAccessFile(file, mode))
    } yield {
      new WKWFile(header, fileMode, underlyingFile)
    }
  }

  def read[T](is: InputStream)(f: (WKWHeader, Iterator[Array[Byte]]) => T): Box[T] = {
    ResourceBox.manage(new DataInputStream(is)) { dataStream =>
      for {
        header <- WKWHeader(dataStream, true)
      } yield {
        val blockIterator = header.blockLengths.flatMap { blockLength =>
          val data = Array.ofDim[Byte](blockLength)
          dataStream.read(data)
          if (header.isCompressed) decompressBlock(header.blockType, header.numBytesPerBlock)(data) else Full(data)
        }
        f(header, blockIterator)
      }
    }
  }

  def write(os: OutputStream, header: WKWHeader, blocks: Iterator[Array[Byte]]): Box[Unit] = {
    val dataBuffer = new ByteArrayOutputStream()
    (0 until header.numBlocksPerCube).foldLeft[Box[Array[Int]]](Full(Array.emptyIntArray)) {
      case (Full(blockLengths), _) =>
        if (blocks.hasNext) {
          val data = blocks.next
          for {
            _ <- Check(data.length == header.numBytesPerBlock) ?~! error("Unexpected block size", header.numBytesPerBlock, data.length)
            compressedBlock <- if (header.isCompressed) compressBlock(header.blockType)(data) else Full(data)
            _ <- Try(dataBuffer.write(compressedBlock))
          } yield {
            blockLengths :+ compressedBlock.length
          }
        } else {
          Failure("No more blocks in iterator.")
        }
      case (f, _) =>
        f
    }.map { blockLengths =>
      val jumpTable = if (header.isCompressed) blockLengths.map(_.toLong).scan(header.dataOffset)(_ + _) else Array(header.dataOffset)
      header.copy(jumpTable = jumpTable).writeTo(new DataOutputStream(os))
      dataBuffer.flush()
      dataBuffer.writeTo(os)
      dataBuffer.close()
    }
  }
}
