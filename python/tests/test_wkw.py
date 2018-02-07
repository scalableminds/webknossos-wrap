import wkw
import numpy as np
import shutil
from os import path

POSITION = (128, 128, 128)
SIZE = (32, 32, 32)


def test_context_manager():
    wkw.Dataset.create('tests/tmp', wkw.Header(np.uint16)).close()

    with wkw.Dataset.open('tests/tmp') as dataset:
        assert dataset.header.voxel_type == np.uint16

def test_create():
    with wkw.Dataset.create('tests/tmp', wkw.Header(np.uint8)) as dataset:
        assert dataset.header.voxel_type == np.uint8
        assert path.exists(path.join('tests/tmp', 'header.wkw'))

def test_open():
    wkw.Dataset.create('tests/tmp', wkw.Header(np.uint16)).close()

    with wkw.Dataset.open('tests/tmp') as dataset:
        assert dataset.header.voxel_type == np.uint16


def test_readwrite():
    with wkw.Dataset.create('tests/tmp', wkw.Header(np.uint8)) as dataset:

        header_size = path.getsize(path.join('tests/tmp', 'header.wkw'))
        test_data = generate_test_data(dataset.header.voxel_type)

        dataset.write(POSITION, test_data)
        assert path.getsize(path.join('tests/tmp', 'z0', 'y0', 'x0.wkw')) == \
               np.prod(SIZE) * (dataset.header.file_len ** 3) + header_size
        assert np.all(dataset.read(POSITION, SIZE) == test_data)


def test_compress():
    with wkw.Dataset.create('tests/tmp', wkw.Header(np.uint8)) as dataset:

        test_data = generate_test_data(dataset.header.voxel_type)
        dataset.write(POSITION, test_data)
    
        with dataset.compress('tests/tmp2', compress_files=True) as dataset2:
            assert dataset2.header.voxel_type == np.uint8
            assert dataset2.header.block_type == wkw.Header.BLOCK_TYPE_LZ4HC

            header_size = path.getsize(path.join('tests/tmp2', 'header.wkw'))

            assert path.exists(path.join('tests/tmp2', 'header.wkw'))
            assert path.getsize(path.join('tests/tmp2', 'z0', 'y0', 'x0.wkw')) < \
                   np.prod(SIZE) * (dataset2.header.file_len ** 3) + header_size
            assert np.all(dataset2.read(POSITION, SIZE) == test_data)


def generate_test_data(dtype):
    return np.random.uniform(0, 255, SIZE).astype(dtype)


def try_rmtree(dir):
    try:
        shutil.rmtree(dir)
    except FileNotFoundError:
        pass


def teardown_function():
    try_rmtree('tests/tmp')
    try_rmtree('tests/tmp2')



