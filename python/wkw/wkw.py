import os
import ctypes
import numpy as np
import cffi

def __init_libwkw():
    this_dir = os.path.dirname(__file__)
    path_wkw_header = os.path.join(this_dir, 'lib', 'wkw.h')
    path_wkw_lib = os.path.join(this_dir, 'lib', 'libwkw.so')

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
                 voxel_size: int,
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

class Dataset:
    def __init__(self, root, handle):
        self.root = root
        self.handle = handle

        header_c = ffi.new("struct header *")
        libwkw.dataset_get_header(self.handle, header_c)
        self.header = Header.from_c(header_c)

    def read(self, bbox):
        assert isinstance(bbox, np.ndarray)
        assert bbox.dtype == np.uint32
        assert bbox.shape == (3, 2)

        data_shape = np.append(self.header.num_channels,
                               bbox[:, 1] - bbox[:, 0])
        data = np.zeros(data_shape,
                        dtype=self.header.voxel_type,
                        order='F')

        bbox_f = np.asfortranarray(bbox)
        bbox_ptr = ffi.cast("uint32_t *", bbox_f.ctypes.data)
        data_ptr = ffi.cast("void *", data.ctypes.data)
        libwkw.dataset_read(self.handle, bbox_ptr, data_ptr)

        return data

    def write(self, off, data):
        assert isinstance(off, np.ndarray)
        assert off.dtype == np.uint32
        assert off.size == 3

        assert isinstance(data, np.ndarray)
        assert data.dtype == self.header.voxel_type

        bbox = np.zeros((3, 2), dtype='uint32', order='F')
        bbox_ptr = ffi.cast("uint32_t *", bbox.ctypes.data)

        bbox[:, 0] = off
        bbox[:, 1] = off + data.shape

        data_f = np.asfortranarray(data)
        data_ptr = ffi.cast("void *", data_f.ctypes.data)
        C.dataset_write(self.handle, bbox_ptr, data_ptr)

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
    def __raise_wkw_exception():
        error_msg = ffi.string(libwkw.get_last_error_msg())
        raise WKWException(error_msg.decode())

    def __enter__(self):
        return self

    def __exit__(self, type, value, tb):
        self.close()
