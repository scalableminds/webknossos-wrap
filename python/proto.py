import os
import ctypes
import numpy as np
from cffi import FFI

ffi = FFI()
ffi.cdef("""
    struct header {
        uint8_t version;
        uint8_t block_len;
        uint8_t file_len;
        uint8_t block_type;
        uint8_t voxel_type;
        uint8_t voxel_size;
    };

    typedef struct dataset dataset_t;

    void * dataset_open(const char * root);
    void   dataset_close(const dataset_t * handle);
    void   dataset_read(const dataset_t * handle, uint32_t * bbox, void * data);
    void   dataset_get_header(const dataset_t * handle, struct header * header);
    char * get_last_error_msg();
""")

this_dir = os.path.dirname(__file__)
path_libwkw = os.path.join(this_dir, '../c/target/debug/libwkw.so')
C = ffi.dlopen(path_libwkw)

class WKWException(Exception):
    pass

class Header:
    BLOCK_TYPE_RAW = 1
    BLOCK_TYPE_LZ4 = 2
    BLOCK_TYPE_LZ4HC = 3

    VALID_BLOCK_TYPES = [BLOCK_TYPE_RAW, BLOCK_TYPE_LZ4, BLOCK_TYPE_LZ4HC]
    VALID_VOXEL_TYPES = [np.uint8, np.uint16, np.uint32, np.uint64, np.float32, np.float64]

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
        self.numel = voxel_size // np.dtype(voxel_type).itemsize
        self.voxel_size = voxel_size

    @staticmethod
    def from_c(header_c):
        print(header_c)

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
    def __init__(self, root: str, handle):
        self.root = root
        self.handle = handle

        header_c = ffi.new("struct header *")
        C.dataset_get_header(self.handle, header_c)
        self.header = Header.from_c(header_c)

    def read(self, bbox):
        assert isinstance(bbox, np.ndarray)
        assert bbox.dtype == np.uint32
        assert bbox.shape == (3, 2)

        data = np.zeros(bbox[:, 1] - bbox[:, 0],
                        dtype=self.header.voxel_type,
                        order='F')

        bbox_f = np.asfortranarray(bbox)
        bbox_ptr = ffi.cast("uint32_t *", bbox_f.ctypes.data)
        data_ptr = ffi.cast("void *", data.ctypes.data)
        C.dataset_read(self.handle, bbox_ptr, data_ptr)

        return data

    def close(self):
        C.dataset_close(self.handle)

    @staticmethod
    def open(root: str):
        root_c = ffi.new("char[]", root.encode())
        handle = C.dataset_open(root_c)

        if handle == ffi.NULL:
            error_msg = ffi.string(C.get_last_error_msg())
            raise WKWException(error_msg)
        else:
            return Dataset(root, C.dataset_open(root_c))

    def __enter__(self):
        return self

    def __exit__(self, type, value, tb):
        self.close()

root = "/home/amotta/Desktop/wkw"
with Dataset.open(root) as dataset:
    header = dataset.header
    
    print("Header:")
    print("  block_len: ", header.block_len)
    print("  file_len: ", header.file_len)
    print("  block_type: ", header.block_type)
    print("  voxel_type: ", header.voxel_type)
    print("  voxel_size: ", header.voxel_size)
    
    bbox = np.zeros((3, 2), dtype='uint32', order='F')
    bbox[:, 0] = [10, 20, 30]
    bbox[:, 1] = [70, 60, 50]
    
    data = dataset.read(bbox)
    print("data: ", data)
