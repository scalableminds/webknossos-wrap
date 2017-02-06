/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap.util

import net.liftweb.common.{Box, Failure, Full}
import resource._
import com.scalableminds.webknossos.wrap.util.BoxHelpers._

object ResourceBox {
  def apply[R : Resource](resource: => R): Box[R] = {
    Try(resource) ~> "Exception during resource creation"
  }

  def manage[R : Resource, T](resource: => R)(f: R => Box[T]): Box[T] = {
    for {
      r <- ResourceBox(resource)
      result <- managed(r).map(f).either.either match {
        case Left(ex) =>
          Failure(s"Exception during resource management: ${ex.toString}")
        case Right(result) =>
          result
      }
    } yield {
      result
    }
  }
}
