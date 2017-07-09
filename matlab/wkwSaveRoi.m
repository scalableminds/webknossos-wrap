function wkwSaveRoi(filePath, offset, data) %#ok
    % wkwSaveRoi(filePath, offset, data)
    %   Saves a three-dimensional data volume to a WKW dataset.
    %
    % filePath
    %   String. Absolute path to root of WKW dataset.
    %
    % offset
    %   1x3 double. Target position of the first voxel.
    %
    % data
    %   KxLxM or CxKxLxM <T>. Three- or four-dimensional matrix with data
    %   to save. If the matrix is four-dimensional, the first dimension
    %   must correspond to the number of channels in the WKW dataset.
    %
    % Example
    %   data = randi([0, intmax('uint8')], [123, 234, 345], 'uint8');
    %   wkwSave('/gaba/u/amotta/wkw', [129, 0, 257], data);
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    error('Please run wkwBuild to compile wkwSaveRoi');
end
