/*
* Copyright (C) 2011-2017 Scalable minds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
*/
package com.scalableminds.webknossos.wrap.util

import com.scalableminds.webknossos.wrap.util.BoxHelpers._
import java.io.RandomAccessFile
import java.nio.MappedByteBuffer

import net.liftweb.common.{Box, Failure}
import org.apache.commons.lang3.reflect.FieldUtils

object ExtendedTypes {

  implicit class ExtendedRandomAccessFile(f: RandomAccessFile) {
    def isClosed: Boolean = {
      val method = f.getClass.getDeclaredField("closed")
      method.setAccessible(true)
      method.getBoolean(f)
    }

    def getPath: String = {
      val method2 = f.getClass.getDeclaredField("path")
      method2.setAccessible(true)
      method2.get(f).asInstanceOf[String]
    }
  }

  implicit class ExtendedMappedByteBuffer(mappedData: MappedByteBuffer) {
    private val unsafe = FieldUtils.readField(mappedData, "unsafe", true)

    private val address = FieldUtils.readField(mappedData, "address", true).asInstanceOf[Long]

    private  val arrayBaseOffset = FieldUtils.readField(mappedData, "arrayBaseOffset", true).asInstanceOf[Long]

    private val unsafeCopy = {
      val m = unsafe.getClass.getDeclaredMethod("copyMemory",
        classOf[Object], classOf[Long], classOf[Object], classOf[Long], classOf[Long])
      m.setAccessible(true)
      m
    }

    def copyTo(offset: Long, other: Array[Byte], destPos: Long, length: java.lang.Integer): Box[Unit] = {
      // Any regularly called log statements in here should be avoided as they drastically slow down this method.
      if (offset + length < mappedData.limit()) {
        Try(() => {
          val memOffset: java.lang.Long = address + offset
          val targetOffset: java.lang.Long = destPos + arrayBaseOffset
          // Anything that might go south here can result in a segmentation fault, so be careful!
          unsafeCopy.invoke(unsafe, null, memOffset, other, targetOffset, length)
        })
      } else {
        Failure("Could not copy from memory mapped array.")
      }
    }
  }
}
