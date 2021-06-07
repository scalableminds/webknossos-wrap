package com.scalableminds.webknossos.wrap.util

import java.io.RandomAccessFile
import java.lang.reflect.Method
import java.nio.MappedByteBuffer

import net.liftweb.common.{Box, Failure, Full}
import org.apache.commons.lang3.reflect.FieldUtils
import net.liftweb.util.Helpers.tryo

object ExtendedTypes {

  implicit class ExtendedRandomAccessFile(f: RandomAccessFile) {
    private val closedF = {
      val method = f.getClass.getDeclaredField("closed")
      method.setAccessible(true)
      method
    }

    private val pathF = {
      val method = f.getClass.getDeclaredField("path")
      method.setAccessible(true)
      method
    }

    def isClosed: Boolean = closedF.getBoolean(f)

    def getPath: String = pathF.get(f).asInstanceOf[String]
  }

  class ExtendedMappedByteBuffer(mappedData: MappedByteBuffer) {

    def capacity: Int = mappedData.capacity

    def copyTo(offset: Long, other: Array[Byte], destPos: Long, length: java.lang.Integer): Box[Unit] = {
      // Any regularly called log statements in here should be avoided as they drastically slow down this method.
      if (offset + length <= mappedData.limit()) {
        tryo {
          for (i <- 0 until length) {
            other(destPos.toInt + i) = mappedData.get(offset.toInt + i)
          }
          Full(())
        }
      } else {
        Failure("Could not copy from memory mapped array.")
      }
    }

    def copyFrom(offset: Long, other: Array[Byte], srcPos: Long, length: java.lang.Integer): Box[Unit] = {
      // Any regularly called log statements in here should be avoided as they drastically slow down this method.
      if (offset + length <= mappedData.limit()) {
        tryo {
          for (i <- 0 until length) {
            mappedData.put(offset.toInt + i, other(srcPos.toInt + i))
          }
          Full(())
        }
      } else {
        Failure("Could not copy to memory mapped array.")
      }
    }
  }
}
