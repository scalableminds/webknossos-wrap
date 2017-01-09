function boxes = mortonSplitBox(box, cubeSize)
    % config
    minLoadSize = 128;
    
    % prepare output
    boxes = zeros(3, 2, 0);
    boxWidth = diff(box, 1, 2);
    
    % sanity checks
    assert(all(box(:, 1) >= 0));
    assert(all(boxWidth >= 0));
    
    % check if cube is already in agreement with Morton
    if all(boxWidth <= cubeSize) && ~any(diff(boxWidth)) && ...
       all(isPowerOfTwo(boxWidth)) && ~any(rem(box(:, 1), boxWidth))
        boxes = box;
        return;
    end
    
    % enlarge box to fix with minimum load size
    box(:, 1) = minLoadSize .* floor(box(:, 1) ./ minLoadSize);
    box(:, 2) = minLoadSize .* ceil(box(:, 2) ./ minLoadSize);
    
    % decompose it into three-dimensional cubes
    for curSizeLog2 = fliplr(log2(minLoadSize):log2(cubeSize))
        curSize = 2 ^ curSizeLog2;
        
        curIds = [ ...
            ceil(box(:, 1) ./ curSize), ...
            floor(box(:, 2) ./ curSize) - 1];
        
        curBoxes = nan([6, diff(curIds, 1, 2)' + 1]);
        [curBoxes(1, :, :, :), curBoxes(2, :, :, :), curBoxes(3, :, :, :)] = ...
            ndgrid( ...
                curSize .* (curIds(1, 1):curIds(1, 2)), ...
                curSize .* (curIds(2, 1):curIds(2, 2)), ...
                curSize .* (curIds(3, 1):curIds(3, 2)));
        curBoxes = reshape(curBoxes, 3, 2, []);
        
        curBoxesDone = arrayfun(@(idx) ...
            inBoxes(boxes, curBoxes(:, 1, idx)), 1:size(curBoxes, 3));
        
        % filter and complete bounding boxes
        curBoxes = curBoxes(:, :, not(curBoxesDone));
        curBoxes(:, 2, :) = curBoxes(:, 1, :) + curSize;
        
        % complete list of boxes
        boxes = cat(3, boxes, curBoxes);
    end
end

function flag = inBoxes(boxes, coordVec)
    flag = any( ...
        all(bsxfun(@ge, coordVec(:), boxes(:, 1, :)), 1) ...
      & all(bsxfun(@lt, coordVec(:), boxes(:, 2, :)), 1));
end

function flag = isPowerOfTwo(vals)
    flag = false(size(vals));
    flag(vals == 0) = true;
    flag(vals ~= 0) = ~rem(log2(vals), 1);
end