function wkwFromKnossos(wkParam, wkwRoot, dataType)
    % wkwFromKnossos(wkParam, wkwRoot, dataType)
    %   Converts a KNOSSOS hierarchy into wk-wrap files.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    assert(ismember(dataType, {'uint8', 'uint32'}));
    
    % find resolutions
    resolutions = findResolutions(wkParam.root);
    
    forResolution = @(r) ...
        wkwFromKnossosResolution(wkParam, wkwRoot, dataType);
    arrayfun(forResolution, resolutions);
end

function resolutions = findResolutions(wkRoot)
    fprintf('Searching resolutions... ');
    
    % find available resolutions
    [~, resDirs] = getFilesAndDirs(wkRoot);
    resDirs = resDirs(cellfun(@all, isstrprop(resDirs, 'digit')));
    resolutions = cellfun(@str2num, resDirs);
    
    % make sure that "high-res" data is available
    resolutions = sort(resolutions);
    assert(~isempty(resolutions) && resolutions(1) == 1);
    
    disp('✔');
    disp(['  Found ', strjoin( ...
        arrayfun(@num2str, resolutions, 'Uni', false), ', ')]);
end

function [wkParam, wkwRoot] = ...
        buildParametersForResolution(wkParam, wkwRoot, resolution)
    % webKNOSSOS parameters
    wkParam.root = fullfile(wkParam.root, num2str(resolution));
    wkParam.prefix = [wkParam.prefix, '_mag', num2str(resolution)];
    
    % NOTE: For historical reasons, the read- / writeKnossosRoi functions
    % expect the root directory to end with a directory separator
    wkParam.root = strcat(wkParam.root, filesep);
    
    % webKNOSSOS wrap parameters
    wkwRoot = fullfile(wkwRoot, num2str(resolution));
end

function wkwFromKnossosResolution(wkParam, wkwRoot, dataType, resolution)
    % config
    fileClen = 1024;
    
    % update parameters
    [wkParam, wkwRoot] = ...
        buildParametersForResolution(wkParam, wkwRoot, resolution);
    
    disp(['Processing ', wkParam.prefix, '...']);
    fprintf('  Determining bounding box... ');
    box = getBoundingBox(wkParam.root);
    disp('✔');
    
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
    
    fprintf('  Converting... ');
    job = Cluster.startJob( ...
        @wkwFromKnossosFile, jobInputs, ...
        'sharedInputs', jobSharedInputs, ...
        'cluster', cluster, ...
        'name', mfilename);
    
    wait(job);
    disp('✔');
end

function wkwFromKnossosFile(wkParam, wkwRoot, dataType, box)
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

function [files, dirs] = getFilesAndDirs(root)
    dirData = dir(root);
    dirEntries = {dirData.name};
    
    % build masks
    dirMask = [dirData.isdir];
    visMask = ~strncmp('.', dirEntries, 1);
    
    % build output
    files = dirEntries(visMask & ~dirMask);
    dirs  = dirEntries(visMask &  dirMask);
end

function files = getAllFiles(wkRoot)
    % list directory
    [files, subDirs] = getFilesAndDirs(wkRoot);
    
    % recurse into subdirectories
    subDirs = fullfile(wkRoot, subDirs);
    subDirs = cellfun(@getAllFiles, subDirs, 'UniformOutput', false);
    
    % build complete file list
    files = vertcat(fullfile(wkRoot, files(:)), subDirs{:});
end