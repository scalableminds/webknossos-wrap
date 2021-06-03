import cffi
import ctypes
import numpy as np
import platform
import os
from copy import deepcopy
from glob import iglob


def _init_libwkw():
    this_dir = os.path.dirname(__file__)
    path_wkw_header = os.path.join(this_dir, "lib", "wkw.h")

    lib_name_platform = {
        "Linux": "libwkw.so",
        "Windows": "wkw.dll",
        "Darwin": "libwkw.dylib",
    }
    path_wkw_lib = os.path.join(this_dir, "lib", lib_name_platform[platform.system()])

    with open(path_wkw_header) as f:
        wkw_header = f.readlines()

        # strip away directives to be compatible with cffi module
        wkw_header = filter(lambda l: not l.startswith("#"), wkw_header)
        wkw_header = "\n".join(wkw_header)

    ffi = cffi.FFI()
    ffi.cdef(wkw_header)
    libwkw = ffi.dlopen(path_wkw_lib)

    return (ffi, libwkw)


ffi, libwkw = _init_libwkw()


class WKWException(Exception):
    pass


def _raise_wkw_exception():
    error_msg = ffi.string(libwkw.get_last_error_msg())
    raise WKWException(error_msg.decode())


def _check_wkw(ret):
    if ret > 0:
        _raise_wkw_exception()


def _check_wkw_null(ret):
    if ret == ffi.NULL:
        _raise_wkw_exception()
    return ret


class Header:
    BLOCK_TYPE_RAW = 1
    BLOCK_TYPE_LZ4 = 2
    BLOCK_TYPE_LZ4HC = 3

    VALID_BLOCK_TYPES = [BLOCK_TYPE_RAW, BLOCK_TYPE_LZ4, BLOCK_TYPE_LZ4HC]
    VALID_VOXEL_TYPES = [
        np.uint8,
        np.uint16,
        np.uint32,
        np.uint64,
        np.float32,
        np.float64,
        np.int8,
        np.int16,
        np.int32,
        np.int64,
    ]

    def __init__(
        self,
        voxel_type: type,
        num_channels: int = 1,
        version: int = 1,
        block_len: int = 32,
        file_len: int = 32,
        block_type: int = 1,
    ):
        self.version = version

        assert block_len & (block_len - 1) == 0
        self.block_len = block_len

        assert file_len & (file_len - 1) == 0
        self.file_len = file_len

        assert block_type in self.VALID_BLOCK_TYPES
        self.block_type = block_type

        assert voxel_type in self.VALID_VOXEL_TYPES
        self.voxel_type = voxel_type

        assert num_channels > 0
        self.num_channels = num_channels

    @staticmethod
    def from_c(header_c):
        assert header_c.voxel_type > 0
        voxel_type = Header.VALID_VOXEL_TYPES[header_c.voxel_type - 1]
        voxel_type_size = np.dtype(voxel_type).itemsize

        assert header_c.voxel_size % voxel_type_size == 0
        num_channels = header_c.voxel_size // voxel_type_size

        assert header_c.block_type > 0
        block_type = Header.VALID_BLOCK_TYPES[header_c.block_type - 1]

        return Header(
            version=header_c.version,
            block_len=header_c.block_len,
            file_len=header_c.file_len,
            block_type=block_type,
            voxel_type=voxel_type,
            num_channels=num_channels,
        )

    def to_c(self):
        voxel_type_c = Header.VALID_VOXEL_TYPES.index(self.voxel_type) + 1
        voxel_type_size = np.dtype(self.voxel_type).itemsize
        voxel_size = self.num_channels * voxel_type_size

        header_c = ffi.new("struct header *")
        header_c.version = self.version
        header_c.file_len = self.file_len
        header_c.block_len = self.block_len
        header_c.block_type = self.block_type
        header_c.voxel_type = voxel_type_c
        header_c.voxel_size = voxel_size
        return header_c


def _build_box(off, shape):
    assert off[0] >= 0, "Offset x must be greater or equal to 0"
    assert off[1] >= 0, "Offset y must be greater or equal to 0"
    assert off[2] >= 0, "Offset z must be greater or equal to 0"
    assert shape[0] >= 0, "Shape x must be greater or equal to 0"
    assert shape[1] >= 0, "Shape y must be greater or equal to 0"
    assert shape[2] >= 0, "Shape z must be greater or equal to 0"

    off = np.asarray(off, dtype=np.uint32)
    shape = np.asarray(shape, dtype=np.uint32)
    return np.hstack((off, off + shape))


class File:
    @staticmethod
    def compress(src_path: str, dst_path: str):
        src_path_c = ffi.new("char[]", src_path.encode())
        dst_path_c = ffi.new("char[]", dst_path.encode())

        _check_wkw(libwkw.file_compress(src_path_c, dst_path_c))


class Dataset:
    def __init__(self, root, handle):
        self.root = ffi.string(root).decode("utf-8")
        self.handle = handle

        header_c = ffi.new("struct header *")
        libwkw.dataset_get_header(self.handle, header_c)
        self.header = Header.from_c(header_c)

    def read(self, off, shape):
        box = _build_box(off, shape)
        box_ptr = ffi.cast("uint32_t *", box.ctypes.data)

        num_channels = self.header.num_channels
        data = np.zeros(
            (num_channels,) + tuple(shape), order="F", dtype=self.header.voxel_type
        )
        data_ptr = ffi.cast("void *", data.ctypes.data)

        _check_wkw(libwkw.dataset_read(self.handle, box_ptr, data_ptr))
        return data

    def write(self, off, data):
        if not isinstance(data, np.ndarray):
            raise WKWException("Data must be a NumPy ndarray")

        if not data.ndim in [3, 4]:
            raise WKWException("Data must be three- or four-dimensional")

        data = data.reshape((-1,) + data.shape[-3:])
        if not data.shape[0] == self.header.num_channels:
            raise WKWException(
                "Data volume must have {} channels".format(self.header.num_channels)
            )

        if not data.dtype == self.header.voxel_type:
            raise WKWException(
                "Data elements must be of type {}".format(self.header.voxel_type)
            )

        def is_contiguous(data):
            return data.flags["F_CONTIGUOUS"] or data.flags["C_CONTIGUOUS"]

        if not is_contiguous(data):
            data = np.asfortranarray(data)

        box = _build_box(off, data.shape[-3:])
        box_ptr = ffi.cast("uint32_t *", box.ctypes.data)

        assert is_contiguous(data), "Input data is not contiguous"

        data_in_c_order = data.flags["C_CONTIGUOUS"]
        data_ptr = ffi.cast("void *", data.ctypes.data)
        _check_wkw(
            libwkw.dataset_write(self.handle, box_ptr, data_ptr, data_in_c_order)
        )

    def compress(self, dst_path: str, compress_files: bool = False):
        header = deepcopy(self.header)
        header.block_type = Header.BLOCK_TYPE_LZ4HC

        dataset = Dataset.create(dst_path, header)

        if compress_files:
            for file in self.list_files():
                rel_file = os.path.relpath(file, self.root)
                File.compress(file, os.path.join(dst_path, rel_file))

        return dataset

    def list_files(self):
        return iglob(os.path.join(self.root, "*", "**", "*.wkw"))

    def close(self):
        libwkw.dataset_close(self.handle)

    @staticmethod
    def open(root: str, header=None):
        root_c = ffi.new("char[]", root.encode())
        handle = None
        if header is not None:
            try:
                handle = _check_wkw_null(libwkw.dataset_create(root_c, header.to_c()))
            except WKWException:
                handle = _check_wkw_null(libwkw.dataset_open(root_c))
        else:
            handle = _check_wkw_null(libwkw.dataset_open(root_c))
        return Dataset(root_c, handle)

    @staticmethod
    def create(root: str, header):
        root_c = ffi.new("char[]", root.encode())
        handle = _check_wkw_null(libwkw.dataset_create(root_c, header.to_c()))
        return Dataset(root_c, handle)

    def __enter__(self):
        return self

    def __exit__(self, type, value, tb):
        self.close()
