name := "webknossos-wrap"

scalaVersion := "2.12.7"

javaOptions in test ++= Seq("-Xmx512m")

javacOptions ++= Seq("-source", "1.8", "-target", "1.8")

scalacOptions ++= Seq("-unchecked", "-deprecation")

scalacOptions in (Compile, doc) ++= Seq(
  "-unchecked",
  "-deprecation",
  "-implicits"
)

publishMavenStyle := true

publishArtifact in Test := false

pomIncludeRepository := { _ => false }

publishTo := {
  val nexus = "https://oss.sonatype.org/"
  if (isSnapshot.value) Some("snapshots" at nexus + "content/repositories/snapshots")
  else Some("releases" at nexus + "service/local/staging/deploy/maven2")
}

organization := "com.scalableminds"

organizationName := "scalable minds GmbH"

organizationHomepage := Some(url("http://scalableminds.com"))

startYear := Some(2017)

description := "A small library to load webknossos-wrap encoded files."

homepage := Some(url("https://github.com/scalableminds/webknossos-wrap"))

licenses := Seq("MIT" -> url("https://github.com/scalableminds/webknossos-wrap/blob/master/LICENSE"))

scmInfo := Some(ScmInfo(
  url("https://github.com/scalableminds/webknossos-wrap"),
  "https://github.com/scalableminds/webknossos-wrap.git"))

pomExtra := (
  <developers>
    <developer>
      <id>fm3</id>
      <name>Florian M</name>
      <url>https://github.com/fm3</url>
    </developer>
  </developers>
)

libraryDependencies ++= Seq(
  "com.google.guava" % "guava" % "21.0",
  "com.jsuereth" %% "scala-arm" % "2.0",
  "com.newrelic.agent.java" % "newrelic-agent" % "3.31.1",
  "com.newrelic.agent.java" % "newrelic-api" % "3.31.1",
  "com.typesafe.scala-logging" %% "scala-logging" % "3.5.0",
  "net.jpountz.lz4" % "lz4" % "1.3.0",
  "net.liftweb" % "lift-common_2.10" % "2.6-M3",
  "net.liftweb" % "lift-util_2.10" % "3.0-M1",
  "org.apache.commons" % "commons-lang3" % "3.1",
  "commons-io" % "commons-io" % "2.4",
  "org.apache.logging.log4j" % "log4j-api" % "2.0-beta9",
  "org.apache.logging.log4j" % "log4j-core" % "2.0-beta9"
)

releasePublishArtifactsAction := PgpKeys.publishSigned.value

val root = (project in file("."))
  .enablePlugins(BuildInfoPlugin)
  .settings(
    buildInfoKeys := Seq[BuildInfoKey](name, version, scalaVersion, sbtVersion,
      "commitHash" -> new java.lang.Object() {
        override def toString(): String = {
          try {
            val extracted = new java.io.InputStreamReader(java.lang.Runtime.getRuntime().exec("git rev-parse HEAD").getInputStream())
            (new java.io.BufferedReader(extracted)).readLine()
          } catch {
            case t: Throwable => "get git hash failed"
          }
        }
      }.toString()
    ),
    buildInfoPackage := "webknossoswrap",
    buildInfoOptions := Seq(BuildInfoOption.ToJson, BuildInfoOption.BuildTime)
  )
