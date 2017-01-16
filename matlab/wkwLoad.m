function data = wkwLoad(filePath, size, offset, class) %#ok
    % data = wkwLoad(filePath, size, offset, class)
    %   Loads a three-dimensional data cube from a Morton-encoded binary
    %   file and decodes it to Fortran-order for use in MATLAB.
    %
    % filePath
    %   String. Absolute path to the Morton-encoded file.
    %
    % size
    %   Scalar. Side length of the data cube to be loaded. By design of the
    %   Morton encoding, this value must be a power of two!
    %
    % offset
    %   Three-dimensional vector. Position of the first data element
    %   relative to the entire Morton cube. Note that the offset must
    %   be aligned to cubes of the requested size.
    %
    % class
    %   String. Class of the individual data elements. For now, only uint8
    %   and uint32 are supported.
    %
    % Example
    %   raw = wkwLoad( ...
    %       '/home/amotta/raw.dat', 128, [257, 129, 513], 'uint8');
    %   seg = wkwLoad( ...
    %       '/home/amotta/seg.dat', 128, [257, 129, 513], 'uint32');
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    error('Please run wkwBuild to compile wkwLoad.cpp');
end
