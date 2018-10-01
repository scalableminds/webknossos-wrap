import wkw
import numpy as np
import shutil
from os import path
import pytest

POSITION = (0, 0, 0)
POSITION2 = (64, 64, 64)
POSITION3 = (1024, 512, 1024)
SIZE = (32, 32, 32)
SIZE2 = (128, 128, 128)
SIZE3 = (256, 256, 256)


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


def test_readwrite_live_compression():
    with wkw.Dataset.create('tests/tmp', wkw.Header(np.uint8, block_type=wkw.Header.BLOCK_TYPE_LZ4)) as dataset:

        header_size = path.getsize(path.join('tests/tmp', 'header.wkw'))
        test_data = generate_test_data(dataset.header.voxel_type)
        test_data2 = generate_test_data(dataset.header.voxel_type, SIZE2)
        test_data3 = generate_test_data(dataset.header.voxel_type, SIZE3)

        dataset.write(POSITION, test_data)

        # The size should be less than if it was not compressed
        assert path.getsize(path.join('tests/tmp', 'z0', 'y0', 'x0.wkw')) < \
               np.prod(SIZE) * (dataset.header.file_len ** 3) + header_size

        dataset.write(POSITION2, test_data2)
        dataset.write(POSITION3, test_data3)

    with wkw.Dataset.open('tests/tmp') as dataset:
        assert np.all(dataset.read(POSITION2, SIZE2) == test_data2)
        assert np.all(dataset.read(POSITION, SIZE) == test_data)
        assert np.all(dataset.read(POSITION3, SIZE3) == test_data3)


def test_readwrite_live_compression_should_enforce_morton_order():
    with pytest.raises(Exception):
        with wkw.Dataset.create('tests/tmp', wkw.Header(np.uint8, block_type=BLOCK_TYPE_LZ4)) as dataset:

            test_data = generate_test_data(dataset.header.voxel_type)

            dataset.write(POSITION2, test_data)
            # Should fail since POSITION has a lower morton-order rank than POSITION2
            dataset.write(POSITION, test_data)


def test_should_not_crash_when_skipping_blocks_with_live_compression():
    dataset = wkw.Dataset.create('tests/tmp', wkw.Header(np.uint8, block_type=wkw.Header.BLOCK_TYPE_LZ4))

    test_data = np.empty((128, 128, 128)).astype(np.uint8)

    # test_data_slice_a = np.random.uniform(0, 255, (64, 64, 64)).astype(np.uint8)
    test_data_slice_b = np.ones((64, 128, 64)).astype(np.uint8)

    # test_data[:64, :64, :64] = test_data_slice_a
    test_data[64:128, :128, :64] = test_data_slice_b

    # dataset.write((0, 0, 0), test_data_slice_a)
    dataset.write((64, 0, 0), test_data_slice_b)

    # assert np.all(dataset.read((0, 0, 0), (64, 64, 64)) == test_data_slice_a)
    # assert np.all(dataset.read((64, 0, 0), (64, 128, 64)) == test_data_slice_b)
    read_data = dataset.read((0, 0, 0), (128, 128, 128))

    print("test data", test_data[4:8, 4:8, 64:68])
    print("read data", read_data[4:8, 4:8, 64:68])

    assert np.all(read_data == test_data)


def test_should_not_crash_when_reading_first_block():
    dataset = wkw.Dataset.create('tests/tmp', wkw.Header(np.uint8, block_type=wkw.Header.BLOCK_TYPE_LZ4))
    test_data = np.empty((128, 128, 128)).astype(np.uint8)
    test_data_slice_b = np.random.uniform(0, 255, (64, 128, 64)).astype(np.uint8)

    test_data[64:128, :128, :64] = test_data_slice_b
    dataset.write((64, 0, 0), test_data_slice_b)

    read_data = dataset.read((0, 0, 0), (128, 128, 128))

    assert np.all(read_data == test_data)


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


def generate_test_data(dtype, size=SIZE):
    return np.random.uniform(0, 255, size).astype(dtype)


def try_rmtree(dir):
    try:
        shutil.rmtree(dir)
    except FileNotFoundError:
        pass


def teardown_function():
    try_rmtree('tests/tmp')
    try_rmtree('tests/tmp2')
