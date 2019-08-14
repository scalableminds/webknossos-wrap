import wkw
import numpy as np
import shutil
from os import path
import pytest

POSITION = (0, 0, 0)
SIZE = (32, 32, 32)


def test_context_manager():
    wkw.Dataset.create("tests/tmp", wkw.Header(np.uint16)).close()

    with wkw.Dataset.open("tests/tmp") as dataset:
        assert dataset.header.voxel_type == np.uint16


def test_create():
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)) as dataset:
        assert dataset.header.voxel_type == np.uint8
        assert path.exists(path.join("tests/tmp", "header.wkw"))


def test_open():
    wkw.Dataset.create("tests/tmp", wkw.Header(np.uint16)).close()

    with wkw.Dataset.open("tests/tmp") as dataset:
        assert dataset.header.voxel_type == np.uint16


def test_readwrite():
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)) as dataset:

        header_size = path.getsize(path.join("tests/tmp", "header.wkw"))
        test_data = generate_test_data(dataset.header.voxel_type)

        dataset.write(POSITION, test_data)
        assert (
            path.getsize(path.join("tests/tmp", "z0", "y0", "x0.wkw"))
            == np.prod(SIZE) * (dataset.header.file_len ** 3) + header_size
        )
        assert np.all(dataset.read(POSITION, SIZE) == test_data)


def test_readwrite_live_compression():
    SIZE128 = (128, 128, 128)
    file_len = 4
    header = wkw.Header(
        np.uint8, block_type=wkw.Header.BLOCK_TYPE_LZ4, file_len=file_len
    )
    with wkw.Dataset.create("tests/tmp", header) as dataset:
        header_size = path.getsize(path.join("tests/tmp", "header.wkw"))
        test_data = generate_test_data(dataset.header.voxel_type, SIZE128)

        dataset.write(POSITION, test_data)

        # The size should be less than if it was not compressed
        assert (
            path.getsize(path.join("tests/tmp", "z0", "y0", "x0.wkw"))
            < np.prod(SIZE128) * (dataset.header.file_len ** 3) + header_size
        )

    with wkw.Dataset.open("tests/tmp") as dataset:
        assert np.all(dataset.read(POSITION, SIZE128) == test_data)


def test_readwrite_live_compression_should_enforce_full_file_write():
    with pytest.raises(Exception):
        with wkw.Dataset.create(
            "tests/tmp", wkw.Header(np.uint8, block_type=BLOCK_TYPE_LZ4)
        ) as dataset:

            test_data = generate_test_data(dataset.header.voxel_type)
            dataset.write(POSITION, test_data)


def test_readwrite_live_compression_should_not_allow_inconsistent_writes():
    SIZE129 = (129, 128, 128)
    file_len = 4
    header = wkw.Header(
        np.uint8, block_type=wkw.Header.BLOCK_TYPE_LZ4, file_len=file_len
    )
    test_data = generate_test_data(header.voxel_type, SIZE129)
    empty_data = np.zeros(SIZE129).astype(header.voxel_type)

    with wkw.Dataset.create("tests/tmp", header) as dataset:
        with pytest.raises(Exception):
            dataset.write(POSITION, test_data)

    with wkw.Dataset.open("tests/tmp") as dataset:
        assert np.all(dataset.read(POSITION, SIZE129) == empty_data)


def test_readwrite_live_compression_should_truncate():
    SIZE128 = (128, 128, 128)
    file_len = 4
    header = wkw.Header(
        np.uint8, block_type=wkw.Header.BLOCK_TYPE_LZ4, file_len=file_len
    )
    test_data = generate_test_data(header.voxel_type, SIZE128)
    ones_data = np.ones(SIZE128).astype(header.voxel_type)

    with wkw.Dataset.create("tests/tmp", header) as dataset:
        dataset.write(POSITION, test_data)

    random_compressed_size = path.getsize(path.join("tests/tmp", "z0", "y0", "x0.wkw"))

    with wkw.Dataset.open("tests/tmp") as dataset:
        dataset.write(POSITION, ones_data)

    empty_compressed_size = path.getsize(path.join("tests/tmp", "z0", "y0", "x0.wkw"))

    assert empty_compressed_size < random_compressed_size

    with wkw.Dataset.open("tests/tmp") as dataset:
        assert np.all(dataset.read(POSITION, SIZE128) == ones_data)


def test_compress():
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)) as dataset:

        test_data = generate_test_data(dataset.header.voxel_type)
        dataset.write(POSITION, test_data)

        with dataset.compress("tests/tmp2", compress_files=True) as dataset2:
            assert dataset2.header.voxel_type == np.uint8
            assert dataset2.header.block_type == wkw.Header.BLOCK_TYPE_LZ4HC

            header_size = path.getsize(path.join("tests/tmp2", "header.wkw"))

            assert path.exists(path.join("tests/tmp2", "header.wkw"))
            assert (
                path.getsize(path.join("tests/tmp2", "z0", "y0", "x0.wkw"))
                < np.prod(SIZE) * (dataset2.header.file_len ** 3) + header_size
            )
            assert np.all(dataset2.read(POSITION, SIZE) == test_data)


def generate_test_data(dtype, size=SIZE):
    return np.random.uniform(0, 255, size).astype(dtype)


def try_rmtree(dir):
    try:
        shutil.rmtree(dir)
    except FileNotFoundError:
        pass


def teardown_function():
    try_rmtree("tests/tmp")
    try_rmtree("tests/tmp2")
