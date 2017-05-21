resolvers ++= Seq(
    DefaultMavenRepository,
    "Typesafe Repository" at "http://repo.typesafe.com/typesafe/releases/"
)

addSbtPlugin("com.typesafe.play" % "sbt-plugin" % "2.4.6")

addSbtPlugin("com.frugalmechanic" % "fm-sbt-s3-resolver" % "0.9.0")

addSbtPlugin("com.github.gseitz" % "sbt-release" % "1.0.3")

addSbtPlugin("com.eed3si9n" % "sbt-buildinfo" % "0.7.0")
