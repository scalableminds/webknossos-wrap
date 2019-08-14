# Build on Windows

* Install [Visual Studio 2019 Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2019) with Windows 10 SDK
* Install [Git](https://git-scm.com/download/win)
* Install [Python 3.7](https://www.python.org/ftp/python/3.7.4/python-3.7.4-amd64.exe). Make sure that Python 2.7 is not also installed
* Install [Rust via Rustup](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)
* Download and extract [lz4](https://github.com/lz4/lz4/releases/download/v1.8.2/lz4_v1_8_2_win64.zip). Version `>1.8.*` doesn't work because there are no bundled dll files
* From `lz4*.zip` copy `dll/liblz4.*` to `c/liblz4.*`
* Rename `liblz4.so.1.8.2.dll` to `liblz4.dll`
* Go to `python` and run `python setup.py install`
* Test with `python -c "import wkw"`
* `pip install wheel`
* Write credentials to `~/.pypirc`
* `python setup.py bdist_wheel -p win-amd64 upload`
