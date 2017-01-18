import sbt._
import sbt.Keys._
import play.sbt.Play.autoImport._
import play.sbt.routes.RoutesKeys._

object BuildSettings {
  val buildVersion = scala.io.Source.fromFile("version").mkString.trim

  val scalaCompilerVersion = "2.11.7"

  val filter = { (ms: Seq[(File, String)]) =>
    ms filter {
      case (file, path) =>
        path != "logback.xml" && !path.startsWith("toignore") && !path.startsWith("samples")
    }
  }

  val buildSettings = Defaults.defaultSettings ++ Seq(
    version := buildVersion,
    scalaVersion := scalaCompilerVersion,
    javaOptions in test ++= Seq("-Xmx512m", "-XX:MaxPermSize=512m"),
    javacOptions ++= Seq("-source", "1.7", "-target", "1.7"),
    scalacOptions ++= Seq("-unchecked", "-deprecation" /*, "-Xlog-implicits", "-Yinfer-debug", "-Xprint:typer", "-Yinfer-debug", "-Xlog-implicits", "-Xprint:typer"*/ ),
    scalacOptions in (Compile, doc) ++= Seq("-unchecked", "-deprecation", "-implicits"),
    shellPrompt := ShellPrompt.buildShellPrompt,
    mappings in (Compile, packageBin) ~= filter,
    mappings in (Compile, packageSrc) ~= filter,
    mappings in (Compile, packageDoc) ~= filter) ++ Publish.settings
}

object Publish {
  object TargetRepository {
    def scmio: Def.Initialize[Option[sbt.Resolver]] = version { (version: String) =>
      val rootDir = "/srv/maven/"
      val path =
        if (version.trim.endsWith("SNAPSHOT"))
          "snapshots"
        else
          "releases"
      Some("scm.io intern repo" at "s3://maven.scm.io.s3-eu-central-1.amazonaws.com/" + path)
    }
  }
  lazy val settings = Seq(
    organization := "com.scalableminds",
    publishMavenStyle := true,
    publishTo <<= TargetRepository.scmio,
    publishArtifact in Test := false,
    pomIncludeRepository := { _ => false },
    homepage := Some(url("http://scm.io")))
}

object Colors {

  import scala.Console._

  lazy val isANSISupported = {
    Option(System.getProperty("sbt.log.noformat")).map(_ != "true").orElse {
      Option(System.getProperty("os.name"))
        .map(_.toLowerCase)
        .filter(_.contains("windows"))
        .map(_ => false)
    }.getOrElse(true)
  }

  def red(str: String): String = if (isANSISupported) (RED + str + RESET) else str
  def blue(str: String): String = if (isANSISupported) (BLUE + str + RESET) else str
  def cyan(str: String): String = if (isANSISupported) (CYAN + str + RESET) else str
  def green(str: String): String = if (isANSISupported) (GREEN + str + RESET) else str
  def magenta(str: String): String = if (isANSISupported) (MAGENTA + str + RESET) else str
  def white(str: String): String = if (isANSISupported) (WHITE + str + RESET) else str
  def black(str: String): String = if (isANSISupported) (BLACK + str + RESET) else str
  def yellow(str: String): String = if (isANSISupported) (YELLOW + str + RESET) else str

}

// Shell prompt which show the current project,
// git branch and build version
object ShellPrompt {
  object devnull extends ProcessLogger {
    def info(s: => String) {}

    def error(s: => String) {}

    def buffer[T](f: => T): T = f
  }

  def currBranch = (
    ("git status -sb" lines_! devnull headOption)
    getOrElse "-" stripPrefix "## ")

  val buildShellPrompt = {
    (state: State) =>
      {
        val currProject = Project.extract(state).currentProject.id
        ("%s "+ Colors.green("(%s)") + ": %s> ").format(
          currProject, currBranch, BuildSettings.buildVersion)
      }
  }
}

object Resolvers {
  val resolversList = Seq(
      "repo.novus rels" at "http://repo.novus.com/releases/",
      "repo.novus snaps" at "http://repo.novus.com/snapshots/",
      "sonatype rels" at "https://oss.sonatype.org/content/repositories/releases/",
      "sonatype snaps" at "https://oss.sonatype.org/content/repositories/snapshots/",
      "sgodbillon" at "https://bitbucket.org/sgodbillon/repository/raw/master/snapshots/",
      //"mandubian" at "https://github.com/mandubian/mandubian-mvn/raw/master/snapshots/",
      "typesafe" at "http://repo.typesafe.com/typesafe/releases",
      Resolver.url("Scalableminds REL Repo", url("http://scalableminds.github.com/releases/"))(Resolver.ivyStylePatterns)
    )
}

object Dependencies {
  val braingamesVersion = "9.0.5"
  val braingamesUtil = "com.scalableminds" %% "util" % braingamesVersion

  val scalaLogging = "com.typesafe.scala-logging" %% "scala-logging" % "3.4.0"
  val liftBox = "net.liftweb" % "lift-common_2.10" % "2.6-M3"
  val snappy = "org.xerial.snappy" % "snappy-java" % "1.1.2.1"

  val log4jVersion = "2.0-beta9"
  val log4jCore =  "org.apache.logging.log4j" % "log4j-api" % log4jVersion
  val log4jApi = "org.apache.logging.log4j" % "log4j-core" % log4jVersion

  val newrelic = "com.newrelic.agent.java" % "newrelic-agent" % "3.31.1"
  val newrelicApi = "com.newrelic.agent.java" % "newrelic-api" % "3.31.1"
}

object WebknossosWrap extends Build {
  import BuildSettings._
  import Resolvers._
  import Dependencies._

  val webknossosWrap = Project(
    "webknossos-wrap",
    file("webknossos-wrap"),
    settings = buildSettings ++ Seq(
      resolvers := resolversList,
      libraryDependencies ++= Seq(
        braingamesUtil,
        newrelic,
        newrelicApi,
        log4jCore,
        log4jApi,
        scalaLogging,
        liftBox)))

  /* val libs = Project(
    "braingames-libs",
    file("."),
    settings = buildSettings ++ Seq(
      publish := {},
      publishLocal := {}
    )) aggregate(util, binary, datastore) */
}
