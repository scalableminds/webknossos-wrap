/*
* Copyright (C) 2011-2017 Scalable minds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
*/
package com.scalableminds.webknossos.wrap.util

import com.google.common.io.LittleEndianDataOutputStream
import java.io.{DataOutputStream, RandomAccessFile}

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

  implicit class ExtendedLittleEndianDataOutputStream(o: LittleEndianDataOutputStream) {
    def size: Int = {
      val outField = o.getClass.getSuperclass.getDeclaredField("out")
      outField.setAccessible(true)
      val out = outField.get(o).asInstanceOf[DataOutputStream]
      out.size()
    }
  }

}
