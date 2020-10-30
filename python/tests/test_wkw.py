import wkw
import numpy as np
import shutil
from os import path, makedirs
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


def test_non_negative_offsets():
    wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)).close()

    with pytest.raises(AssertionError):
        with wkw.Dataset.open("tests/tmp") as dataset:
            dataset.read((-1, 0, 0), (0, 0, 0))

    with pytest.raises(AssertionError):
        with wkw.Dataset.open("tests/tmp") as dataset:
            dataset.read((0, -1, 0), (0, 0, 0))

    with pytest.raises(AssertionError):
        with wkw.Dataset.open("tests/tmp") as dataset:
            dataset.read((0, 0, -1), (0, 0, 0))

    with pytest.raises(AssertionError):
        with wkw.Dataset.open("tests/tmp") as dataset:
            dataset.read((0, 0, 0), (-1, -1, -1))


def test_empty_read():
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)) as dataset:

        data = dataset.read((1, 1, 1), (0, 0, 0))
        assert data.shape == (1, 0, 0, 0)


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


def test_row_major_order():
    data_shape = (4, 5, 6)
    data = generate_test_data(np.uint8, data_shape)
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)) as dataset:
        dataset.write((0, 0, 0), data)
        read_data = dataset.read((0, 0, 0), data_shape)

    assert np.all(data == read_data)

    with wkw.Dataset.create("tests/tmp2", wkw.Header(np.uint8)) as dataset:
        fortran_data = np.asfortranarray(data)
        dataset.write((0, 0, 0), fortran_data)
        fortran_read_data = dataset.read((0, 0, 0), data_shape)

    assert np.all(fortran_read_data == read_data)
    assert np.all(fortran_read_data == fortran_data)


def test_row_major_order_with_offset():
    data_shape = (17, 1, 4)
    data = generate_test_data(np.uint8, data_shape)
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)) as dataset:
        dataset.write((15, 2, 0), data)
        read_data = dataset.read((15, 2, 0), data_shape)

    assert np.all(data == read_data)


def test_row_major_order_with_different_voxel_size():
    data_shape = (4, 3, 9)
    data = generate_test_data(np.uint16, data_shape)
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint16)) as dataset:
        dataset.write((3, 1, 0), data)
        read_data = dataset.read((3, 1, 0), data_shape)

    assert np.all(data == read_data)


def test_row_major_order_with_channels():
    data_shape = (2, 4, 3, 9)
    data = generate_test_data(np.uint8, data_shape)
    with wkw.Dataset.create(
        "tests/tmp", wkw.Header(np.uint8, num_channels=2)
    ) as dataset:
        dataset.write((3, 1, 0), data)
        read_data = dataset.read((3, 1, 0), data_shape[1:])

    assert np.all(data == read_data)


def test_row_major_order_with_channels_and_different_voxel_size():
    data_shape = (2, 4, 3, 9)
    data = generate_test_data(np.uint16, data_shape)
    with wkw.Dataset.create(
        "tests/tmp", wkw.Header(np.uint16, num_channels=2)
    ) as dataset:
        dataset.write((3, 1, 0), data)
        read_data = dataset.read((3, 1, 0), data_shape[1:])

    assert np.all(data == read_data)


def test_column_major_order_with_channels_and_different_voxel_size():
    data_shape = (2, 4, 3, 9)
    data = generate_test_data(np.uint16, data_shape, order="F")
    with wkw.Dataset.create(
        "tests/tmp", wkw.Header(np.uint16, num_channels=2)
    ) as dataset:
        dataset.write((3, 1, 0), data)
        read_data = dataset.read((3, 1, 0), data_shape[1:])

    assert np.all(data == read_data)


def test_view_on_np_array():
    data_shape = (4, 4, 9)
    data = generate_test_data(np.uint16, data_shape)
    data = data[:, ::2]
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint16)) as dataset:
        dataset.write((3, 1, 0), data)
        read_data = dataset.read((3, 1, 0), data.shape)

    assert np.all(data == read_data)


def test_not_too_much_data_is_written():
    def write_and_test_in_given_order(wkw_path, order):
        data_shape = (35, 35, 35)
        data = generate_test_data(np.uint8, data_shape, order=order)
        with wkw.Dataset.create(wkw_path, wkw.Header(np.uint8)) as dataset:
            dataset.write((0, 0, 0), np.ones((35, 35, 64), dtype=np.uint8))
            dataset.write((1, 2, 3), data)

            read_data = dataset.read((1, 2, 3), (35, 35, 35))
            before = dataset.read((0, 0, 0), (1, 2, 3))
            after = dataset.read((0, 0, 38), (35, 35, 26))

        assert np.all(data == read_data)
        assert np.all(before == 1)
        assert np.all(after == 1)

    write_and_test_in_given_order("tests/tmp", "F")
    write_and_test_in_given_order("tests/tmp2", "C")


def test_multiple_writes_and_reads():

    mem_buffer = np.zeros((200, 200, 200), dtype=np.uint8, order="F")
    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)) as dataset:
        for i in range(10):
            offset = np.random.randint(100, size=(3))
            size = np.random.randint(1, 100, size=(3))
            order = np.random.choice(["F", "C"])
            data = generate_test_data(np.uint8, [1] + list(size), order)
            dataset.write(offset, data)
            mem_buffer[
                offset[0] : offset[0] + size[0],
                offset[1] : offset[1] + size[1],
                offset[2] : offset[2] + size[2],
            ] = data

            read_data = dataset.read((0, 0, 0), (200, 200, 200))
            assert np.all(mem_buffer == read_data)


def test_multi_channel_column_major_order():

    with wkw.Dataset.create(
        "tests/tmp", wkw.Header(np.uint8, num_channels=3)
    ) as dataset:
        offset = (30, 20, 10)
        data_shape = (3, 100, 200, 300)
        order = "C"
        data = generate_test_data(np.uint8, list(data_shape), order)
        dataset.write(offset, data)

        read_data = dataset.read(offset, data_shape[1:])
        assert np.all(data == read_data)


def test_big_read():
    data = np.ones((10, 10, 764), order="C", dtype=np.uint8)
    offset = np.array([0, 0, 640])
    bottom = (1200, 2000, 2000)

    with wkw.Dataset.create("tests/tmp", wkw.Header(np.uint8)) as dataset:
        dataset.write(offset, data)
        read_data = dataset.read((0, 0, 0), bottom)[0]
        assert np.all(
            read_data[
                offset[0] : (offset[0] + data.shape[0]),
                offset[1] : (offset[1] + data.shape[1]),
                offset[2] : (offset[2] + data.shape[2]),
            ]
            == 1
        )
        assert np.count_nonzero(read_data[: offset[0], :, :]) == 0
        assert np.count_nonzero(read_data[offset[0] + data.shape[0] :, :, :]) == 0
        assert np.count_nonzero(read_data[:, : offset[1], :]) == 0
        assert np.count_nonzero(read_data[:, offset[1] + data.shape[1] :, :]) == 0
        assert np.count_nonzero(read_data[:, :, : offset[2]]) == 0
        assert np.count_nonzero(read_data[:, :, offset[2] + data.shape[2] :]) == 0


def test_invalid_dataset():
    with pytest.raises(wkw.wkw.WKWException) as excinfo:
        with wkw.Dataset.open("/path/does/not/exist") as dataset:
            pass
    print(excinfo.value)

    with pytest.raises(wkw.wkw.WKWException) as excinfo:
        makedirs("tests/tmp/exists", exist_ok=True)
        with wkw.Dataset.open("tests/tmp/exists") as dataset:
            pass
    print(excinfo.value)


def generate_test_data(dtype, size=SIZE, order="C"):
    return np.array(
        np.random.uniform(np.iinfo(dtype).min, np.iinfo(dtype).max, size).astype(dtype),
        order=order,
    )


def try_rmtree(dir):
    try:
        shutil.rmtree(dir)
    except FileNotFoundError:
        pass


def setup_function():
    np.random.seed(0)


def teardown_function():
    try_rmtree("tests/tmp")
    try_rmtree("tests/tmp2")
