/*
* Copyright (C) 2011-2017 Scalable minds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
*/
package com.scalableminds.webknossos.wrap.util

import com.scalableminds.webknossos.wrap.util.BoxHelpers._
import java.io.RandomAccessFile
import java.nio.MappedByteBuffer
import net.liftweb.common.{Box, Empty, Failure, Full}
import org.apache.commons.lang3.reflect.FieldUtils

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
    val unsafe = FieldUtils.readField(mappedData, "unsafe", true)

    val address = FieldUtils.readField(mappedData, "address", true).asInstanceOf[Long]

    val arrayBaseOffset = FieldUtils.readField(mappedData, "arrayBaseOffset", true).asInstanceOf[Long]

    val unsafeCopy = {
      val m = unsafe.getClass.getDeclaredMethod("copyMemory",
        classOf[Object], classOf[Long], classOf[Object], classOf[Long], classOf[Long])
      m.setAccessible(true)
      m
    }

    def capacity = mappedData.capacity

    def copyTo(offset: Long, other: Array[Byte], destPos: Long, length: java.lang.Integer): Box[Unit] = {
      // Any regularly called log statements in here should be avoided as they drastically slow down this method.
      if (offset + length < mappedData.limit()) {
        Try {
          val memOffset: java.lang.Long = address + offset
          val targetOffset: java.lang.Long = destPos + arrayBaseOffset
          // Anything that might go south here can result in a segmentation fault, so be careful!
          unsafeCopy.invoke(unsafe, null, memOffset, other, targetOffset, length)
          Full(())
        }
      } else {
        Failure("Could not copy from memory mapped array.")
      }
    }

    def copyFrom(offset: Long, other: Array[Byte], srcPos: Long, length: java.lang.Integer): Box[Unit] = {
      // Any regularly called log statements in here should be avoided as they drastically slow down this method.
      if (offset + length < mappedData.limit()) {
        Try {
          val memOffset: java.lang.Long = address + offset
          val srcOffset: java.lang.Long = srcPos + arrayBaseOffset
          // Anything that might go south here can result in a segmentation fault, so be careful!
          unsafeCopy.invoke(unsafe, other, srcOffset, null, memOffset, length)
          Full(())
        }
      } else {
        Failure("Could not copy to memory mapped array.")
      }
    }
  }
}
