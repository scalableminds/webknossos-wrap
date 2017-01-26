name := "webknossos-wrap"

scalaVersion := "2.11.7"

javaOptions in test ++= Seq("-Xmx512m")

javacOptions ++= Seq("-source", "1.7", "-target", "1.7")

scalacOptions ++= Seq("-unchecked", "-deprecation")

scalacOptions in (Compile, doc) ++= Seq(
  "-unchecked",
  "-deprecation",
  "-implicits"
)

publishMavenStyle := true

publishArtifact in Test := false

pomIncludeRepository := { _ => false }

publishTo <<= version { (version: String) =>
  val rootDir = "/srv/maven/"
  val path =
    if (version.trim.endsWith("SNAPSHOT"))
      "snapshots"
    else
      "releases"
  Some("scm.io intern repo" at "s3://maven.scm.io.s3-eu-central-1.amazonaws.com/" + path)
}

organization := "com.scalableminds"

organizationName := "scalable minds UG (haftungsbeschränkt) & Co. KG"

organizationHomepage := Some(url("http://scalableminds.com"))

startYear := Some(2017)

description := "A small library to load webknossos-wrap encoded files."

homepage := Some(url("https://github.com/scalableminds/webknossos-wrap"))

scmInfo := Some(ScmInfo(
  url("https://github.com/scalableminds/webknossos-wrap"),
  "https://github.com/scalableminds/webknossos-wrap.git"))

libraryDependencies ++= Seq(
  "com.google.guava" % "guava" % "21.0",
  "com.newrelic.agent.java" % "newrelic-agent" % "3.31.1",
  "com.newrelic.agent.java" % "newrelic-api" % "3.31.1",
  "org.apache.logging.log4j" % "log4j-api" % "2.0-beta9",
  "org.apache.logging.log4j" % "log4j-core" % "2.0-beta9",
  "com.typesafe.scala-logging" %% "scala-logging" % "3.4.0",
  "net.liftweb" % "lift-common_2.10" % "2.6-M3"
)