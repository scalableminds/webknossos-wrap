function data = wkwLoadRoi(rootDir, box, dataType)
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    % config
    blockSize = 32;
    cubeSize = 1024;

    % sanity check
    assert(all(box(:) > 0));
    assert(all(box(:, 2) > box(:, 1)));

    % fix bounding box
    % a) make start indices zero-based
    % b) make end indices exclusive
    box(:, 1) = box(:, 1) - 1;

    % find the boxes to load
    boxes = wkwSplitBox(box, blockSize, cubeSize);
    boxCount = size(boxes, 3);

    boxWidth = diff(box, 1, 2);
    data = zeros(boxWidth', dataType);

    % load boxes
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
        curDestLim = bsxfun(@minus, curLimits, box(:, 1) - 1);
        curSrcLim = bsxfun(@minus, curLimits, curBox(:, 1) - 1);

        % read data
        if exist(curFilePath, 'file')
            curSize = diff(curBox(1, :));
            curOffset = 1 + mod(curBox(:, 1), cubeSize);

            curData = wkwLoad( ...
                curFilePath, curSize, curOffset, dataType);

            % cut out relevant part
            if any(curValidBox(:) ~= curBox(:))
                curData = curData( ...
                    curSrcLim(1, 1):curSrcLim(1, 2), ...
                    curSrcLim(2, 1):curSrcLim(2, 2), ...
                    curSrcLim(3, 1):curSrcLim(3, 2));
            end
        else
            curData = 0;
        end

        % write to correct position
        if all(curValidBox(:) == box(:))
            data(:, :, :) = curData;
        else
            data( ...
                curDestLim(1, 1):curDestLim(1, 2), ...
                curDestLim(2, 1):curDestLim(2, 2), ...
                curDestLim(3, 1):curDestLim(3, 2)) = curData;
        end
    end
end
