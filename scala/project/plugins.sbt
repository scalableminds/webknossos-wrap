resolvers ++= Seq(
    DefaultMavenRepository,
    "Typesafe Repository" at "http://repo.typesafe.com/typesafe/releases/"
)

addSbtPlugin("com.typesafe.play" % "sbt-plugin" % "2.4.6")

addSbtPlugin("com.frugalmechanic" % "fm-sbt-s3-resolver" % "0.9.0")
