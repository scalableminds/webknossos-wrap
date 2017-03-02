/*
* Copyright (C) 2011-2017 Scalable minds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
*/
package com.scalableminds.webknossos.wrap.util

import net.liftweb.common.{Box, Empty, Failure, Full}

object BoxHelpers {
  def Check(expression: => Boolean): Box[Unit] = {
    if (expression) {
      Full(Unit)
    } else {
      Failure("Check failed.")
    }
  }

  def Try[T](f: => T): Box[T] = {
    try {
      Full(f)
    } catch {
      case ex: Exception =>
        Failure(s"Unhandled exception: ${ex.getMessage}", Full(ex), Empty)
    }
  }

  def Try[T](f: Box[T]): Box[T] = {
    try {
      f
    } catch {
      case ex: Exception =>
        Failure(s"Unhandled exception: ${ex.getMessage}", Full(ex), Empty)
    }
  }
}
