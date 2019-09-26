function box = wkwBoundingBox(wkwDir)
    % box = wkwBoundingBox(wkwDir)
    %   Returns the tightest possible block-aligned bounding box around the
    %   volume covered by WKW files. Note that the content of the WKW files
    %   itself is ignored, such that all-zero WKW files still contribute to
    %   volume coverage.
    %
    % wkwDir
    %   String. Path to a WKW dataset.
    %
    % box
    %   3x2 matrix. The first and second columns corresponds to the lowest
    %   and highest coordinates contained in the covered volume,
    %   respectively.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    % Load file size from header
    header = wkwLoadHeader(wkwDir);
    voxelsPerFileDim = header.voxelsPerBlockDim * header.blocksPerFileDim;
    
    wkwFiles = dir(fullfile(wkwDir, 'z*', 'y*', 'x*.wkw'));
    if isempty(wkwFiles); box = nan(3, 2); return; end
    
    wkwFiles = arrayfun(@(f) ...
        fullfile(f.folder, f.name), ...
        wkwFiles, 'UniformOutput', false);
    
    escChar = '';
    if strcmp(filesep, '\')
        escChar = '\';
    end
    pattern = [...
        'z(\d+)', filesep, escChar, ...
        'y(\d+)', filesep, escChar, ...
        'x(\d+)'];
    coords = regexp(wkwFiles, pattern, 'tokens', 'once');
    
    coords = flip(cat(1, coords{:}), 2);
    coords = cellfun(@str2double, coords);
    
    box = [min(coords, [], 1)', max(coords, [], 1)'];
    box = (box + [0, 1]) * voxelsPerFileDim + [1, 0];
end
