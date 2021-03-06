function wkwTest()
    %% config
    dataType = 'int8';
    roundCount = 50;
    numChannels = 1;
    blockLen = 32;
    fileLen = 32;
    
    %% preparations
    thisDir = fileparts(mfilename('fullpath'));
    testDir = fullfile(thisDir, 'test');
    
    wkwInit('new', testDir, blockLen, fileLen, dataType, numChannels);
    rmTestDir = onCleanup(@() rmdir(testDir, 's'));
    
    % create RAM matrix
    clen = 1.5 * fileLen * blockLen;
    dataSize = cat(2, numChannels, repmat(clen, 1, 3));
    data = zeros(dataSize, dataType);
    
    % initialize RNG
    rng(0);

    %% run test
    for curIdx = 1:roundCount
        %% write data
        curBox = buildRandBox(clen);
        curData = buildRandDataForBox(dataType, numChannels, curBox);

        % update data
        data( ...
            :, ...
            curBox(1, 1):curBox(1, 2), ...
            curBox(2, 1):curBox(2, 2), ...
            curBox(3, 1):curBox(3, 2)) = curData;
        
        % write to file
        curData = shiftdim(curData, numChannels == 1);
        wkwSaveRoi(testDir, curBox(:, 1)', curData);

        %% read data
        curBox = buildRandBox(clen);
        curWkwData = wkwLoadRoi(testDir, curBox);
        
        curRamData = data( ...
            :, ...
            curBox(1, 1):curBox(1, 2), ...
            curBox(2, 1):curBox(2, 2), ...
            curBox(3, 1):curBox(3, 2));
        curRamData = shiftdim(curRamData, numChannels == 1);

        %% do test
        assert(isequal(size(curWkwData), size(curRamData)));
        assert(all(curWkwData(:) == curRamData(:)));
        disp(['<< Round ', num2str(curIdx), ' passed']);
    end
end

function box = buildRandBox(clen)
    box = nan(3, 2);
    box(:, 1) = randi(clen - 1, 3, 1);
    box(1, 2) = box(1, 1) + randi([1, clen - box(1, 1)]);
    box(2, 2) = box(2, 1) + randi([1, clen - box(2, 1)]);
    box(3, 2) = box(3, 1) + randi([1, clen - box(3, 1)]);
end

function data = buildRandDataForBox(dataType, numChannels, box)
    boxSize = 1 + diff(box, 1, 2)';
    boxSize = cat(2, numChannels, boxSize);
    
    switch dataType
        case { ...
                'int8', 'int16', 'int32', 'int64', ...
                'uint8', 'uint16', 'uint32', 'uint64'}
            dataTypeRange = [intmin(dataType), intmax(dataType)];
            data = randi(dataTypeRange, boxSize, dataType);
        case {'single', 'double'}
            data = rand(boxSize, dataType);
    end
end
