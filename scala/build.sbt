name := "webknossos-wrap"

scalaVersion := "2.13.11"

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
  "com.google.guava" % "guava" % "23.0",
  "org.lz4" % "lz4-java" % "1.8.0",
  "net.liftweb" %% "lift-common" % "3.5.0",
  "net.liftweb" %% "lift-util" % "3.5.0",
  "org.apache.commons" % "commons-lang3" % "3.13.0",
  "commons-io" % "commons-io" % "2.13.0",
)

releasePublishArtifactsAction := PgpKeys.publishSigned.value

val root = (project in file("."))
  .enablePlugins(BuildInfoPlugin)
  .settings(
    buildInfoKeys := Seq[BuildInfoKey](name, version, scalaVersion, sbtVersion,
      "commitHash" -> new java.lang.Object() {
        override def toString: String = {
          try {
            val extracted = new java.io.InputStreamReader(java.lang.Runtime.getRuntime.exec("git rev-parse HEAD").getInputStream)
            new java.io.BufferedReader(extracted).readLine()
          } catch {
            case t: Throwable => "get git hash failed"
          }
        }
      }.toString()
    ),
    buildInfoPackage := "webknossoswrap",
    buildInfoOptions := Seq(BuildInfoOption.ToJson, BuildInfoOption.BuildTime)
  )
