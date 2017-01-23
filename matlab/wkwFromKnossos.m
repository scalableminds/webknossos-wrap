function wkwFromKnossos(wkParam, wkwRoot)
    % wkwFromKnossos(wkParam, wkwRoot)
    %   Converts a KNOSSOS hierarchy into wk-wrap files.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>

    % config
    fileClen = 1024;
    
    % get bounding box
    box = getBoundingBox(wkParam.root);

    % align box with wk-wrap files
    fileIds = [ ...
        floor((box(:, 1) - 1) ./ fileClen), ...
        ceil(box(:, 2) ./ fileClen) - 1];

    % progress
    tic;
    curCount = 1;
    fileCount = prod(diff(fileIds, 1, 2) + 1);

    % copy files
    for curIdxX = fileIds(1, 1):fileIds(1, 2)
        for curIdxY = fileIds(2, 1):fileIds(2, 2)
            for curIdxZ = fileIds(3, 1):fileIds(3, 2)
                % show progress
                Util.progressBar(curCount, fileCount);
                curCount = curCount + 1;

                % box
                curIds = [ ...
                    curIdxX, curIdxY, curIdxZ];
                curBox = [ ...
                    max(box(:, 1),  1 + curIds(:)  .* fileClen), ...
                    min(box(:, 2), (1 + curIds(:)) .* fileClen)];
                curOffset = curBox(:, 1)';

                % do the work
                curData = readKnossosRoi( ...
                    wkParam.root, wkParam.prefix, curBox);
                wkwSaveRoi(wkwRoot, curOffset, curData);
            end
        end
    end
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