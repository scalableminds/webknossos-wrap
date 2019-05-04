function header = wkwLoadHeader(wkwDir)
    % header = wkwLoadHeader(wkwDir)
    %   Loads the header from a WKW header file.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    voxelTypes = { ...
        'uint8', 'uint16', 'uint32', 'uint64', 'single', 'double', ...
        'int8', 'int16', 'int32', 'int64'};
    voxelSizes = [1, 2, 4, 8, 4, 8, 1, 2, 4, 8];
    assert(isequal(size(voxelTypes), size(voxelSizes)));
    
    blockTypes = {'RAW', 'LZ4', 'LZ4HC'};
    
    % Header from WKW file
    wkwHeaderFile = fullfile(wkwDir, 'header.wkw');
    wkwHeaderFd = fopen(wkwHeaderFile, 'r');
    assert(wkwHeaderFd ~= -1, 'Could not open %s', wkwHeaderFile);
    
    wkwHeader = fread(wkwHeaderFd, 16, 'uint8');
    wkwHeader = reshape(wkwHeader, 1, []);
    
    assert(fclose(wkwHeaderFd) == 0);
    clear wkwHeaderFd;
    
    assert( ...
        strcmp(char(wkwHeader(1:3)), 'WKW'), ...
        'Mismatch of magic bytes in %s', wkwHeaderFile);
    
    version = double(wkwHeader(4));
    assert(version == 1, 'Version mismatch in %s', wkwHeaderFile);
    
    perDimLog2 = wkwHeader(5);
    voxelsPerBlockDim = bitand(perDimLog2, uint8(15));
    voxelsPerBlockDim = bitshift(uint16(1), voxelsPerBlockDim);
    blocksPerFileDim = bitand(perDimLog2, uint8(255) - uint8(15));
    blocksPerFileDim = bitshift(uint16(1), bitshift(blocksPerFileDim, -4));
    
    blockType = wkwHeader(6);
    assert( ...
        1 <= blockType && blockType <= numel(blockTypes), ...
        'Invalid block type id %d in %s', blockType, wkwHeaderFile);
    blockType = blockTypes{blockType};
    
    voxelType = wkwHeader(7);
    assert( ...
        1 <= voxelType && voxelType <= numel(voxelTypes), ...
        'Invalid voxel type id %d in %s', voxelType, wkwHeaderFile);
    voxelSize = voxelSizes(voxelType);
    voxelType = voxelTypes{voxelType};
    
    numChannels = double(wkwHeader(8)) / voxelSize;
    assert(mod(numChannels, 1) == 0);
    
    header = struct;
    header.version = double(wkwHeader(4));
    header.voxelsPerBlockDim = double(voxelsPerBlockDim);
    header.blocksPerFileDim = double(blocksPerFileDim);
    header.blockType = blockType;
    header.voxelType = voxelType;
    header.numChannels = numChannels;
end
