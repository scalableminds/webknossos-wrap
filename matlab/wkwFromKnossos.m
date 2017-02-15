function wkwFromKnossos(wkParam, wkwRoot, dataType, box)
    % wkwFromKnossos(wkParam, wkwRoot, dataType)
    %   Converts a KNOSSOS hierarchy into wk-wrap files.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>

    % config
    fileClen = 1024;
    
    % check input
    assert(ismember(dataType, {'uint8', 'uint32'}));
    
    if ~exist('box', 'var') || isempty(box)
        disp('<< Determining bounding box...');
        box = getBoundingBox(wkParam.root);
    else
        assert(isequal(size(box), [3, 2]));
    end

    % align box with wk-wrap files
    fileIds = [ ...
        floor((box(:, 1) - 1) ./ fileClen), ...
        ceil(box(:, 2) ./ fileClen) - 1];
    
    [idsX, idsY, idsZ] = ndgrid( ...
        fileIds(1, 1):fileIds(1, 2), ...
        fileIds(2, 1):fileIds(2, 2), ...
        fileIds(3, 1):fileIds(3, 2));
    
    % NOTE
    % Let's not truncate to the bounding box. This has the advantage that
    % we will always read / write full wkw files and padding won't be
    % needed. In this manner we only use the most efficient code path.
    jobInputs = arrayfun(@(x, y, z) {{[ ...
         1 + [x; y; z]  .* fileClen, ...
        (1 + [x; y; z]) .* fileClen]}}, ...
        idsX(:), idsY(:), idsZ(:));
    jobSharedInputs = {wkParam, wkwRoot, dataType};
    
    cluster = Cluster.getCluster( ...
        '-l h_vmem=12G', '-l h_rt=0:29:00', '-tc 50');
    job = Cluster.startJob( ...
        @wkwFromKnossosCore, jobInputs, ...
        'sharedInputs', jobSharedInputs, ...
        'cluster', cluster, ...
        'name', mfilename);
    
    wait(job);
end

function wkwFromKnossosCore(wkParam, wkwRoot, dataType, box)
    curData = readKnossosRoi( ...
        wkParam.root, wkParam.prefix, box, dataType);
    wkwSaveRoi(wkwRoot, reshape(box(:, 1), 1, []), curData);
end

function box = getBoundingBox(wkRoot)
    cubeSize = [128, 128, 128];
    
    cubeIds = getKnossosCubeIds(wkRoot);
    minIds = min(cubeIds, [], 1);
    maxIds = max(cubeIds, [], 1);
    
    minVec = 1 + minIds .* cubeSize;
    maxVec = (1 + maxIds) .* cubeSize;
    
    box = nan(3, 2);
    box(:, 1) = minVec;
    box(:, 2) = maxVec;
end

function cubeIds = getKnossosCubeIds(wkRoot)
    % first, let's get all files
    files = getAllFiles(wkRoot);
    
    % all file names must be of the form
    % expName_xDDDD_yDDDD_zDDDD.raw
    pattern = '.*_x(\d+)_y(\d+)_z(\d+)\.raw$';
    cubeIds = regexp(files, pattern, 'tokens', 'once');
    
    % to numeric matrix
    cubeIds = vertcat(cubeIds{:});
    cubeIds = cellfun(@str2num, cubeIds);
end

function files = getAllFiles(wkRoot)
    dirData = dir(wkRoot);
    dirMask = [dirData.isdir];
    dirEntries = {dirData.name};
    clear dirData;
    
    files = dirEntries(~dirMask);
    subDirs = dirEntries(dirMask);
    subDirs(ismember(subDirs, {'.', '..'})) = [];
    
    % recurse into subdirectories
    subDirs = fullfile(wkRoot, subDirs);
    subDirs = cellfun(@getAllFiles, subDirs, 'UniformOutput', false);
    
    % build complete file list
    files = vertcat(fullfile(wkRoot, files(:)), subDirs{:});
end