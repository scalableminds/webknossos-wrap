import numpy as np
import wkw

num_reads = 100
voxel_type = np.uint8
block_len = 32
file_len = 32
version = 2

if __name__ == "__main__":
    header = wkw.Header(voxel_type=voxel_type,
                        block_len=block_len,
                        file_len=file_len,
                        version=version)
    dataset = wkw.Dataset.create('./profile', header)

    # write random data
    for x in range(2):
        for y in range(2):
            for z in range(2):
                data = np.random.bytes((block_len * file_len) ** 3)
                data = np.frombuffer(data, dtype=voxel_type)
                data = data.reshape((block_len * file_len,) * 3, order='F')
                offset = np.array((x, y, z)) * block_len * file_len
                dataset.write(offset, data)

    for read in range(num_reads):
        offset = np.random.randint(block_len * file_len - 1, size=(3,))
        shape = map(np.random.randint, block_len * file_len - offset)
        shape = np.array(list(shape)) + 1

        data = dataset.read(offset, shape)
