function data = mortonSaveRoi(rootDir, filePrefix, offset, data)
    % config
    cubeSize = 1024;
    
    % fix bounding box
    % a) make start indices zero-based
    % b) make end indices exclusive
    box = nan(3, 2);
    box(:, 1) = offset - 1;
    box(:, 2) = box(:, 1) + size(data)';
    
    % find the boxes to load
    boxes = mortonSplitBox(box, cubeSize);
    boxCount = size(boxes, 3);
    
    % class of data
    dataType = class(data);
    
    % write boxes
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
        curSrcBox = bsxfun(@minus, curValidBox, box(:, 1) - 1);
        curDestBox = bsxfun(@minus, curValidBox, curBox(:, 1) - 1);
        curOffset = 1 + mod(curBox(:, 1), cubeSize);
        
        if all(curValidBox(:) == box(:))
            curData = data;
        elseif all(curValidBox(:) == curBox(:))
            curData = data( ...
                curSrcBox(1, 1):curSrcBox(1, 2), ...
                curSrcBox(2, 1):curSrcBox(2, 2), ...
                curSrcBox(3, 1):curSrcBox(3, 2));
        else
            if exist(curFilePath, 'file')
                curSize = diff(curBox(1, :));
                
                % fill up cube with existing data
                curData = mortonLoad( ...
                    curFilePath, curSize, curOffset, dataType);
            else
                curData = zeros(diff(curBox, 1, 2)', dataType);
            end
            
            curData( ...
                curDestBox(1, 1):curDestBox(1, 2), ...
                curDestBox(2, 1):curDestBox(2, 2), ...
                curDestBox(3, 1):curDestBox(3, 2)) = ...
                    data( ...
                        curSrcBox(1, 1):curSrcBox(1, 2), ...
                        curSrcBox(2, 1):curSrcBox(2, 2), ...
                        curSrcBox(3, 1):curSrcBox(3, 2));
        end
        
        % save result
        mortonSave(curFilePath, curData, curOffset);
    end
end