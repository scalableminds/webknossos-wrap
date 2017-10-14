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

    void * dataset_open(const char * root);
    void   dataset_close(const void * dataset);
    void   dataset_read(const void * dataset, uint32_t * bbox, void * data);
    void   dataset_get_header(const void * dataset, struct header * header);
""")

this_dir = os.path.dirname(__file__)
path_libwkw = os.path.join(this_dir, '../c/target/debug/libwkw.so')
C = ffi.dlopen(path_libwkw)

class Dataset:
    def __init__(self, root: str, handle):
        self.root = root
        self.handle = handle

        self.header = ffi.new("struct header *")
        C.dataset_get_header(self.handle, self.header)

    def read(self, bbox):
        data = np.zeros(bbox[:, 1] - bbox[:, 0], dtype='uint8', order='F')
        bbox_ptr = ffi.cast("uint32_t *", bbox.ctypes.data)
        data_ptr = ffi.cast("void *", data.ctypes.data)
        C.dataset_read(self.handle, bbox_ptr, data_ptr)

        return data

    def close(self):
        C.dataset_close(self.handle)

    @staticmethod
    def open(root: str):
        root_c = ffi.new("char[]", root.encode())
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
