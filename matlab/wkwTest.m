function wkwTest()
    %% config
    dataType = 'uint8';
    roundCount = 50;
    clen = 2048;

    %% preparations
    thisDir = fileparts(mfilename('fullpath'));
    testDir = fullfile(thisDir, 'test');
    
    wkwInit(testDir, 32, 32, dataType, 1);
    rmTestDir = onCleanup(@() rmdir(testDir, 's'));
    
    % create RAM matrix
    data = zeros(repmat(clen, 1, 3), dataType);

    %% run test
    for curIdx = 1:roundCount
        %% write data
        curBox = buildRandBox(clen);
        curData = randi( ...
            [intmin(dataType), intmax(dataType)], ...
            1 + diff(curBox, 1, 2)', dataType);

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
