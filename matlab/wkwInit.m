function wkwInit(command, varargin) %#ok
    % wkwInit(command, varargin)
    % wkwInit('new', rootDir, blockLen, fileLen, dataType, numChannels)
    % wkwInit('compress', sourceDir, destinationDir)
    %
    %   Initializes a new WKW dataset. This functions supports multiple
    %   operation modes
    %
    %
    % Creation of a brand new dataset with
    %   wkwInit('new', rootDir, blockLen, fileLen, dataType, numChannels)
    %
    %   rootDir
    %     String. Path to the root of the new WKW dataset.
    %
    %   blockLen
    %     Double. Side length of the Fortran-encoded blocks. Must be a
    %     positive power of two. (Default: 32)
    %
    %   fileLen
    %     Double. Side length of the Morton-ordered cube.  Must be a
    %     positive power of two. (Default: 32)
    %
    %   dataType
    %     String. Datatype of the individual voxels. Can be 'uint8',
    %     'uint32', or 'single' (for now).
    %
    %   numChannels
    %     Double. Number of channels. (Default: 1)
    %
    %
    % Creation of a compressed version of an existing dataset with
    %   wkwInit('compress', sourceDir, destinationDir)
    %
    %   sourceDir
    %     String. Path to the root of the existing WKW dataset.
    %
    %   destinationDir
    %     String. Path to the root of the new, compressed WKW dataset.
    %
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    error('Please run wkwBuild to compile wkwInit');
end
