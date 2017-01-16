function wkwSave(filePath, data, offset)
    % wkwSave(filePath, data, offset)
    %   Encodes a three-dimensional data cube to Morton-order and save the
    %   result in a binary file.
    %
    % filePath
    %   String. Absolute path to the Morton-encoded file.
    %
    % data
    %   Three-dimensional matrix. Due to the design of the Morton code
    %   this matrix must be a cube whose side length is a power of two!
    %   For now, the elements of data must be either uint8 or uint32.
    %
    % offset
    %   Three-dimensional vector. Position of the first data element
    %   relative to the entire Morton cube. The offset must be aligned
    %   to cubes with the size of data.
    %
    % Example
    %   data = randi([0, intmax('uint8')], [128, 128, 128], 'uint8');
    %   wkwSave('/home/amotta/morton.dat', data, [129, 0, 257]);
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    error('Please run wkwBuild to compile wkwSave.cpp');
end
