function data = mortonLoadRoi(rootDir, filePrefix, box, dataType)
    % config
    cubeSize = 1024;
    
    % fix bounding box
    % a) make start indices zero-based
    % b) make end indices exclusive
    box(:, 1) = box(:, 1) - 1;
    
    % find the boxes to load
    boxes = mortonSplitBox(box, cubeSize);
    boxCount = size(boxes, 3);
    
    boxWidth = diff(box, 1, 2);
    data = zeros(boxWidth', dataType);
    
    % load boxes
    for curIdx = 1:boxCount
        curBox = boxes(:, :, curIdx);
        curCube = floor(curBox(:, 1) ./ cubeSize);
        
        % build file path
        curFileName = sprintf( ...
            '%s_x%04u_y%04u_z%04u.dat', ...
            filePrefix, curCube(1), curCube(2), curCube(3));
        curFilePath = fullfile(rootDir, curFileName);
        
        % find regions to copy
        curValidBox = [ ...
            max(box(:, 1), curBox(:, 1)), ...
            min(box(:, 2), curBox(:, 2)) - 1];
        
        % make relative to source and destination
        curDestBox = bsxfun(@minus, curValidBox, box(:, 1) - 1);
        curSrcBox = bsxfun(@minus, curValidBox, curBox(:, 1) - 1);
        
        % read data
        if exist(curFilePath, 'file')
            curSize = diff(curBox(1, :));
            curOffset = 1 + mod(curBox(:, 1), cubeSize);
            
            curData = mortonLoad( ...
                curFilePath, curSize, curOffset, dataType);
            
            if any(curValidBox(:) ~= curBox(:))
                % cut out relevant part
                curData = curData( ...
                    curSrcBox(1, 1):curSrcBox(1, 2), ...
                    curSrcBox(2, 1):curSrcBox(2, 2), ...
                    curSrcBox(3, 1):curSrcBox(2, 2));
            end
        else
            curData = 0;
        end
        
        % write to correct position
        if all(curValidBox(:) == box(:))
            data(:, :, :) = curData;
        else
            data( ...
                curDestBox(1, 1):curDestBox(1, 2), ...
                curDestBox(2, 1):curDestBox(2, 2), ...
                curDestBox(3, 1):curDestBox(3, 2)) = curData;
        end
    end
end