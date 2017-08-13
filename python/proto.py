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
    void   dataset_read(const void * dataset, uint32_t * bbox, void * data);
    void   dataset_get_header(const void * dataset, struct header * header);
""")
C = ffi.dlopen("/home/amotta/Code/webknossos-wrap/c/target/debug/libwkw.so")

root = ffi.new("char[]", b"/home/amotta/Desktop/wkw")
dataset = C.dataset_open(root)

header = ffi.new("struct header *")
C.dataset_get_header(dataset, header)

print("Dataset: ", dataset)
print("Header: ", header)
print("  block_len: ", header.block_len)
print("  file_len: ", header.file_len)
print("  block_type: ", header.block_type)
print("  voxel_type: ", header.voxel_type)
print("  voxel_size: ", header.voxel_size)

import numpy as np
import ctypes

bbox = np.zeros((3, 2), dtype='uint32', order='F')
bbox[:, 0] = [10, 20, 30]
bbox[:, 1] = [70, 60, 50]

data = np.zeros(bbox[:, 1] - bbox[:, 0], dtype='uint8', order='F')

bbox_ptr = ffi.cast("uint32_t *", bbox.ctypes.data)
data_ptr = ffi.cast("void *", data.ctypes.data)
C.dataset_read(dataset, bbox_ptr, data_ptr);

print("data: ", data)
