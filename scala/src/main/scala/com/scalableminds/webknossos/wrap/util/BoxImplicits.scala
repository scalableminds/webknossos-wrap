package com.scalableminds.webknossos.wrap.util
import scala.language.implicitConversions

import net.liftweb.common.{Box, Empty, Full}

trait BoxImplicits {
  implicit def bool2Box(b: Boolean): Box[Unit] = if(b) Full(()) else Empty
}
