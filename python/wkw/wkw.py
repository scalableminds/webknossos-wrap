import os
import ctypes
import numpy as np
import cffi
import platform


def __init_libwkw():
    this_dir = os.path.dirname(__file__)
    path_wkw_header = os.path.join(this_dir, 'lib', 'wkw.h')

    if platform.system() == 'Linux':
        path_wkw_lib = os.path.join(this_dir, 'lib', 'libwkw.so')
    elif platform.system() == 'Windows':
        path_wkw_lib = os.path.join(this_dir, 'lib', 'wkw.dll')
    else:
        path_wkw_lib = os.path.join(this_dir, 'lib', 'libwkw.dylib')

    with open(path_wkw_header) as f:
        wkw_header = f.readlines()

        # strip away directives to be compatible with cffi module
        wkw_header = filter(lambda l: not l.startswith('#'), wkw_header)
        wkw_header = "\n".join(wkw_header)

    ffi = cffi.FFI()
    ffi.cdef(wkw_header)
    libwkw = ffi.dlopen(path_wkw_lib)

    return (ffi, libwkw)


ffi, libwkw = __init_libwkw()


class WKWException(Exception):
    pass


class Header:
    BLOCK_TYPE_RAW = 1
    BLOCK_TYPE_LZ4 = 2
    BLOCK_TYPE_LZ4HC = 3

    VALID_BLOCK_TYPES = [BLOCK_TYPE_RAW, BLOCK_TYPE_LZ4, BLOCK_TYPE_LZ4HC]
    VALID_VOXEL_TYPES = [np.uint8, np.uint16, np.uint32, np.uint64, np.float32,
                         np.float64, np.int8, np.int16, np.int32, np.int64]

    def __init__(self,
                 voxel_type: type,
                 voxel_size: int=1,
                 version: int=1,
                 block_len: int=32,
                 file_len: int=32,
                 block_type: int=1):
        self.version = version

        assert block_len & (block_len - 1) == 0
        self.block_len = block_len

        assert file_len & (file_len - 1) == 0
        self.file_len = file_len

        assert block_type in self.VALID_BLOCK_TYPES
        self.block_type = block_type

        assert voxel_type in self.VALID_VOXEL_TYPES
        self.voxel_type = voxel_type

        assert voxel_size > 0
        assert voxel_size % np.dtype(voxel_type).itemsize == 0
        self.num_channels = voxel_size // np.dtype(voxel_type).itemsize
        self.voxel_size = voxel_size

    @staticmethod
    def from_c(header_c):
        assert header_c.voxel_type > 0
        voxel_type = Header.VALID_VOXEL_TYPES[header_c.voxel_type - 1]

        assert header_c.block_type > 0
        block_type = Header.VALID_BLOCK_TYPES[header_c.block_type - 1]

        return Header(version=header_c.version,
                      block_len=header_c.block_len,
                      file_len=header_c.file_len,
                      block_type=block_type,
                      voxel_type=voxel_type,
                      voxel_size=header_c.voxel_size)

    def to_c(self):
        header_c = ffi.new("struct header *")
        header_c.version = self.version
        header_c.file_len = self.file_len
        header_c.block_len = self.block_len
        header_c.block_type = self.block_type
        header_c.voxel_type = Header.VALID_VOXEL_TYPES.index(
            self.voxel_type) + 1
        header_c.voxel_size = self.voxel_size
        return header_c


def _build_box(off, shape):
    off = np.asarray(off, dtype=np.uint32)
    shape = np.asarray(shape, dtype=np.uint32)
    return np.hstack((off, off + shape))


class Dataset:
    def __init__(self, root, handle):
        self.root = root
        self.handle = handle

        header_c = ffi.new("struct header *")
        libwkw.dataset_get_header(self.handle, header_c)
        self.header = Header.from_c(header_c)

    def read(self, off, shape):
        box = _build_box(off, shape)
        box_ptr = ffi.cast("uint32_t *", box.ctypes.data)

        num_channels = self.header.num_channels
        data = np.zeros((num_channels, ) + tuple(shape),
                        order='F', dtype=self.header.voxel_type)
        data_ptr = ffi.cast("void *", data.ctypes.data)

        libwkw.dataset_read(self.handle, box_ptr, data_ptr)
        return data

    def write(self, off, data):
        if not isinstance(data, np.ndarray):
            raise WKWException("Data must be a NumPy ndarray")

        if not data.dtype == self.header.voxel_type:
            raise WKWException("Data elements must be of type {}"
                               .format(self.header.voxel_type))

        box = _build_box(off, data.shape)
        box_ptr = ffi.cast("uint32_t *", box.ctypes.data)

        data = np.asfortranarray(data)
        data_ptr = ffi.cast("void *", data.ctypes.data)
        libwkw.dataset_write(self.handle, box_ptr, data_ptr)

    def close(self):
        libwkw.dataset_close(self.handle)

    @staticmethod
    def open(root: str):
        root_c = ffi.new("char[]", root.encode())
        handle = libwkw.dataset_open(root_c)

        if handle == ffi.NULL:
            Dataset.__raise_wkw_exception()

        return Dataset(root_c, handle)

    @staticmethod
    def create(root: str, header):
        root_c = ffi.new("char[]", root.encode())
        handle = libwkw.dataset_create(root_c, header.to_c())

        if handle == ffi.NULL:
            Dataset.__raise_wkw_exception()

        return Dataset(root_c, handle)

    @staticmethod
    def __raise_wkw_exception():
        error_msg = ffi.string(libwkw.get_last_error_msg())
        raise WKWException(error_msg.decode())

    def __enter__(self):
        return self

    def __exit__(self, type, value, tb):
        self.close()
