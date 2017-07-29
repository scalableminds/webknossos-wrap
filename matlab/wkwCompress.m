function wkwCompress(srcPath, dstPath) %#ok
    % wkwCompress(srcPath, dstPath)
    %   Compresses the blocks of a WKW file using LZ4-HC.
    %
    % srcPath
    %   String. Path to the input file.
    %
    % dstPath
    %   String. Path to the output file.
    %
    % Example
    %   wkwCompress( ...
    %       '/gaba/u/amotta/wkw-uncompressed/z0/y0/x0.wkw', ...
    %       '/gaba/u/amotta/wkw-compressed/z0/y0/x0.wkw');
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    error('Please run wkwBuild to compile wkwCompress');
end
