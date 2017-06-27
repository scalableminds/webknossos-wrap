function data = wkwLoadRoi(filePath, box) %#ok
    % data = wkwLoadRoi(filePath, box)
    %   Loads a three-dimensional data volume from a WKW dataset.
    %
    % filePath
    %   String. Absolute path to root of WKW dataset.
    %
    % box
    %   3x2 double. Bounding box for region of interest.
    %
    % Example
    %   rootDir = '/gaba/u/amotta/wkw';
    %   box = [1, 128; 128, 256; 1, 123];
    %   
    %   data = wkwLoadRoi(rootDir, box);
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    error('Please run wkwBuild to compile wkwLoadRoi');
end
