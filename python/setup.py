from setuptools import setup, find_packages
from setuptools.command.build_py import build_py

import os
import subprocess
import shutil


class BuildPyCommand(build_py):
    """Modified build command to compile C library."""

    def __build_c_library(self):
        this_dir = os.path.dirname(__file__)
        c_dir = os.path.normpath(os.path.join(this_dir, '..', 'c'))

        # building C library
        subprocess.call(['cargo', 'clean'], cwd=c_dir)
        subprocess.call(['cargo', 'build', '--release'], cwd=c_dir)

        lib_name = 'libwkw.so' # TODO(amotta): make this ready for Windows
        lib_file = os.path.join(c_dir, 'target', 'release', lib_name)

        # copying to lib dir
        lib_dir = os.path.join(this_dir, 'wkw', 'lib')

        if os.path.exists(lib_dir):
            shutil.rmtree(lib_dir)

        os.makedirs(lib_dir)
        shutil.copy(lib_file, os.path.join(lib_dir, lib_name))

    def run(self):
        self.__build_c_library()
        build_py.run(self)


setup(
    name="wkw",
    version="0.1",
    author="Alessandro Motta",
    author_email="alessandro.motta@brain.mpg.de",
    url="https://github.com/scalableminds/webknossos-wrap",
    packages=find_packages(),
    include_package_data=True,
    install_requires=['cffi', 'numpy'],
    cmdclass={'build_py': BuildPyCommand}
)
