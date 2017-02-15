function data = wkwSaveRoi(rootDir, offset, data)
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    % config
    blockSize = 32;
    cubeSize = 1024;

    % sanity checks
    assert(all(offset > 0));

    % fix bounding box
    % a) make start indices zero-based
    % b) make end indices exclusive
    box = nan(3, 2);
    box(:, 1) = offset - 1;
    box(:, 2) = box(:, 1) + size(data)';

    % find the boxes to load
    boxes = wkwSplitBox(box, blockSize, cubeSize);
    boxCount = size(boxes, 3);

    % class of data
    dataType = class(data);

    % write boxes
    for curIdx = 1:boxCount
        curBox = boxes(:, :, curIdx);
        curCube = floor(curBox(:, 1) ./ cubeSize);

        % build file path
        curFileName = wkwBuildFilePath(curCube);
        curFilePath = fullfile(rootDir, curFileName);

        % find regions to copy
        curValidBox = [ ...
            max(box(:, 1), curBox(:, 1)), ...
            min(box(:, 2), curBox(:, 2))];

        % make relative to source and destination
        curLimits = bsxfun(@minus, curValidBox, [0, 1]);
        curSrcLim = bsxfun(@minus, curLimits, box(:, 1) - 1);
        curDestLim = bsxfun(@minus, curLimits, curBox(:, 1) - 1);
        curOffset = 1 + mod(curBox(:, 1), cubeSize);

        if all(curValidBox(:) == box(:))
            curData = data;
        elseif all(curValidBox(:) == curBox(:))
            curData = data( ...
                curSrcLim(1, 1):curSrcLim(1, 2), ...
                curSrcLim(2, 1):curSrcLim(2, 2), ...
                curSrcLim(3, 1):curSrcLim(3, 2));
        else
            if exist(curFilePath, 'file')
                % fill up cube with existing data
                curData = wkwLoad(curFilePath, ...
                    diff(curBox(1, :)), curOffset, dataType);
            else
                % fill in fake zeros
                curData = zeros(diff(curBox, 1, 2)', dataType);
            end
            
            curData( ...
                curDestLim(1, 1):curDestLim(1, 2), ...
                curDestLim(2, 1):curDestLim(2, 2), ...
                curDestLim(3, 1):curDestLim(3, 2)) = ...
                    data( ...
                        curSrcLim(1, 1):curSrcLim(1, 2), ...
                        curSrcLim(2, 1):curSrcLim(2, 2), ...
                        curSrcLim(3, 1):curSrcLim(3, 2));
        end
        
        % create directory, if needed
        curFileDir = fileparts(curFilePath);
        if ~exist(curFileDir, 'dir'); mkdir(curFileDir); end;

        % save result
        wkwSave(curFilePath, curData, curOffset);
    end
end
