function wkwTest()
    %% config
    dataType = 'single';
    roundCount = 50;
    clen = 1536;

    %% preparations
    thisDir = fileparts(mfilename('fullpath'));
    testDir = fullfile(thisDir, 'test');
    
    wkwInit('new', testDir, 32, 32, dataType, 1);
    rmTestDir = onCleanup(@() rmdir(testDir, 's'));
    
    % create RAM matrix
    data = zeros(repmat(clen, 1, 3), dataType);

    %% run test
    for curIdx = 1:roundCount
        %% write data
        curBox = buildRandBox(clen);
        curData = buildRandDataForBox(dataType, curBox);

        % update data
        data( ...
            curBox(1, 1):curBox(1, 2), ...
            curBox(2, 1):curBox(2, 2), ...
            curBox(3, 1):curBox(3, 2)) = curData;

        % write to file
        wkwSaveRoi(testDir, curBox(:, 1)', curData);

        %% read data
        curBox = buildRandBox(clen);
        curWkwData = wkwLoadRoi(testDir, curBox);
        
        curRamData = data( ...
            curBox(1, 1):curBox(1, 2), ...
            curBox(2, 1):curBox(2, 2), ...
            curBox(3, 1):curBox(3, 2));

        %% do test
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

function data = buildRandDataForBox(dataType, box)
    boxSize = 1 + diff(box, 1, 2)';
    
    switch dataType
        case {'uint8', 'uint16', 'uint32', 'uint64'}
            dataTypeRange = [intmin(dataType), intmax(dataType)];
            data = randi(dataTypeRange, boxSize, dataType);
        case {'single', 'double'}
            data = rand(boxSize, dataType);
    end
end
