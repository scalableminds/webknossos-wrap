package com.scalableminds.webknossos.wrap.util

import java.nio.MappedByteBuffer

import net.liftweb.common.{Box, Failure, Full}
import net.liftweb.util.Helpers.tryo

object ExtendedTypes {

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
