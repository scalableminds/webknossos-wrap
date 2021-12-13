webknossos-wrap
===============

Is a library for loading data stored on disk in the webKnossos wrap file format. Currently used in the project webKnossos.

## How to build the library

#### Development
To test changes locally, run `sbt publishLocal` from the scala root directory of the library. The current version in `version.sbt` should already be bumped and marked as `SNAPSHOT`. Then adapt the dependency to webknossos-wrap to this version.

#### Release

- Specify the sonatype credentials in `~/.sbt/<sbt version>/sonatype.sbt`:
```
credentials += Credentials("Sonatype Nexus Repository Manager",
       "oss.sonatype.org",
       "<user>",
       "<password>")
```

- Make sure you have [sbt-pgp](https://github.com/sbt/sbt-pgp) up and running, and have an active and published gpg key.
- Make sure all changes are committed locally
- Change directory to the scala root directory.
- Run `sbt release`. This increments the version automatically, publishes the build to a temporary staging repository and adds git commits and tags.
- Log in to [Sonatype Nexus](https://oss.sonatype.org) and close and release the newly created Staging Repository, compare [this guide](https://central.sonatype.org/pages/releasing-the-deployment.html).
- After some delay (10min to 2h) the package should be synced to [maven](https://mvnrepository.com/artifact/com.scalableminds/webknossos-wrap)
- If all is successful, push the git changes via `git push --follow-tags`.

**Never release a version twice. This screws up all caches and may lead to build errors in the projects depending on the library!**
