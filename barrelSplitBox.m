function boxes = barrelSplitBox(box, blockSize, cubeSize)
    minLoadSize = 128;
    
    boxes = nan(3, 2, 0);
    boxWidth = diff(box, 1, 2);
    
    % sanity checks
    assert(all(box(:, 1) >= 0));
    assert(all(boxWidth >= 0));
    
    % this is the short cut for valid boxes
    if all(boxWidth >= blockSize) ... % at least block size
       && all(boxWidth <= cubeSize) ... % not more than file size
       && all(isPowerOfTwo(boxWidth)) ... % cube length is power of two
       && all(diff(boxWidth) == 0) ... % box is actually a cube
       && all(rem(box(:, 1), boxWidth) == 0) % offset is multiple of cube size
        % box in agreement with file format
        boxes = box;
        return;
    end
    
    fileIds = [ ...
        floor(box(:, 1) ./ cubeSize), ...
        ceil(box(:, 2) ./ cubeSize) - 1];
    
    for fileX = fileIds(1, 1):fileIds(1, 2)
        for fileY = fileIds(2, 1):fileIds(2, 2)
            for fileZ = fileIds(3, 1):fileIds(3, 2)
                curBoxes = barrelSplitFileBox( ...
                    box, [fileX, fileY, fileZ], minLoadSize, cubeSize);
                boxes = cat(3, boxes, curBoxes);
            end
        end
    end
end

function boxes = barrelSplitFileBox(box, cubeIds, minSize, cubeSize)
    relOff = cubeIds(:) .* cubeSize;
    
    % make relative to file
    relBox = bsxfun(@minus, box, relOff);
    relBox = [max(0, relBox(:, 1)), min(cubeSize, relBox(:, 2))];
    maxBox = [zeros(3, 1), repmat(cubeSize, 3, 1)];
    
    boxes = barrelSplitCubeBox(relBox, maxBox, minSize);
    boxes = bsxfun(@plus, boxes, relOff);
end

function boxes = barrelSplitCubeBox(box, maxBox, minSize)
    box = max(box, repmat(maxBox(:, 1), 1, 2));
    box = min(box, repmat(maxBox(:, 2), 1, 2));
    
    % check if we can and have to split further
    if any(box(:) ~= maxBox(:)) && any(diff(maxBox, 1, 2) > minSize)
        newBox = [maxBox(:, 1), mean(maxBox, 2)];
        newBoxMake = @(off) bsxfun( ...
            @plus, newBox, diff(maxBox, 1, 2) ./ 2 .* off(:));
        
        boxes = cat(3, ...
            barrelSplitCubeBox(box, newBoxMake([0, 0, 0]), minSize), ...
            barrelSplitCubeBox(box, newBoxMake([0, 0, 1]), minSize), ...
            barrelSplitCubeBox(box, newBoxMake([0, 1, 0]), minSize), ...
            barrelSplitCubeBox(box, newBoxMake([0, 1, 1]), minSize), ...
            barrelSplitCubeBox(box, newBoxMake([1, 0, 0]), minSize), ...
            barrelSplitCubeBox(box, newBoxMake([1, 0, 1]), minSize), ...
            barrelSplitCubeBox(box, newBoxMake([1, 1, 0]), minSize), ...
            barrelSplitCubeBox(box, newBoxMake([1, 1, 1]), minSize));
    else
        boxes = maxBox;
    end
end

function flag = isPowerOfTwo(vals)
    flag = false(size(vals));
    flag(vals == 0) = true;
    flag(vals ~= 0) = ~rem(log2(vals), 1);
end