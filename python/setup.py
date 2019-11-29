from setuptools import setup, find_packages
from setuptools.command.build_py import build_py

import os
import subprocess
import shutil
import platform


class BuildPyCommand(build_py):
    """Modified build command to compile C library."""

    def __build_c_library(self):
        this_dir = os.path.dirname(__file__)
        c_dir = os.path.normpath(os.path.join(this_dir, "..", "c"))

        # building C library
        subprocess.call(["cargo", "clean"], cwd=c_dir)
        subprocess.call(["cargo", "build", "--release"], cwd=c_dir)

        lz4_dir = os.path.normpath(os.path.join(this_dir, "..", "lz4", "lib"))
        lz4_name_platform = {
            "Linux": "liblz4.so.1",
            "Windows": "liblz4.dll",
            "Darwin": "liblz4.1.dylib",
        }
        lz4_name = lz4_name_platform[platform.system()]

        lib_name_platform = {
            "Linux": "libwkw.so",
            "Windows": "wkw.dll",
            "Darwin": "libwkw.dylib",
        }
        lib_name = lib_name_platform[platform.system()]
        lib_file = os.path.join(c_dir, "target", "release", lib_name)
        header_file = os.path.join(c_dir, "include", "wkw.h")

        # copying to lib dir
        lib_dir = os.path.join(this_dir, "wkw", "lib")

        if os.path.exists(lib_dir):
            shutil.rmtree(lib_dir)

        os.makedirs(lib_dir)
        shutil.copy(os.path.join(lz4_dir, lz4_name), os.path.join(lib_dir, lz4_name))
        shutil.copy(lib_file, os.path.join(lib_dir, lib_name))
        shutil.copy(header_file, os.path.join(lib_dir, "wkw.h"))

    def run(self):
        self.__build_c_library()
        build_py.run(self)


setup(
    name="wkw",
    use_scm_version={"root": ".."},
    setup_requires=["setuptools_scm"],
    author="Alessandro Motta",
    author_email="alessandro.motta@brain.mpg.de",
    url="https://github.com/scalableminds/webknossos-wrap",
    packages=find_packages(),
    include_package_data=True,
    install_requires=["cffi", "numpy"],
    cmdclass={"build_py": BuildPyCommand},
)
