webknossos-wrap
===============

Is a library for loading data stored on disk in the webKnossos wrap file format. Currently used in the project webKnossos.

## How to build the library

#### Development
During development, the library should be published locally using

	sbt publish-local

The packages get placed in .ivy2/local/com/scalableminds. To reference the build in another project you need to specify the version, which is contained in the `version` file, in the build file of the dependent project.

#### Release
To release a new version of the libraries to our internal maven repository, make sure to commit all changes before starting the release procedure. You can use the release script to publish the library

	release 0.3.5

This will build the library, publish it, create a github tag and pushes this tag to the origin repository.

**Never release a version twice. This screws up all caches and may lead to build errors in the projects depending on the library!**
