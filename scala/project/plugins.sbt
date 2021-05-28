resolvers ++= Seq(
    DefaultMavenRepository,
    "Typesafe Repository" at "https://repo.typesafe.com/typesafe/maven-releases/"
)

addSbtPlugin("com.frugalmechanic" % "fm-sbt-s3-resolver" % "0.16.0")

addSbtPlugin("com.github.sbt" % "sbt-pgp" % "2.1.2")

addSbtPlugin("com.github.gseitz" % "sbt-release" % "1.0.9")

addSbtPlugin("com.eed3si9n" % "sbt-buildinfo" % "0.7.0")
