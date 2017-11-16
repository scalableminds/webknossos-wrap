/*
* Copyright (C) 2011-2017 Scalable minds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
*/
package com.scalableminds.webknossos.wrap.util

import net.liftweb.common.{Box, Empty, Full}

trait BoxImplicits {
  implicit def bool2Box(b: Boolean): Box[Unit] = if(b) Full(()) else Empty
}
