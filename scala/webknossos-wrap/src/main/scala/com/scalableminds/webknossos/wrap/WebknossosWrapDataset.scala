/*
 * Copyright (C) 2011-2017 scalableminds UG (haftungsbeschränkt) & Co. KG. <http://scm.io>
 */
package com.scalableminds.webknossos.wrap

import com.newrelic.api.agent.NewRelic
import com.scalableminds.util.cache.LRUConcurrentCache
import com.scalableminds.util.tools.Fox

import scala.concurrent.ExecutionContext.Implicits._

case class WKWDataset()
