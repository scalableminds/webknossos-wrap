function wkwTest()
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    %% config
    dataType = 'single';
    roundCount = 50;
    clen = 1024;

    %% preparations
    thisDir = fileparts(mfilename('fullpath'));
    testDir = fullfile(thisDir, 'test');

    % empty directory, if needed
    if exist(testDir, 'dir'); rmdir(testDir, 's'); end;
    mkdir(testDir); rmTestDir = onCleanup(@() rmdir(testDir, 's'));

    % create RAM matrix
    data = zeros(repmat(clen, 1, 3), dataType);
    
    %% run test
    for curIdx = 1:roundCount
        %% write data
        curBox = buildRandBox(clen);
        curData = buildRandData(curBox, dataType);

        % update data
        data( ...
            curBox(1, 1):curBox(1, 2), ...
            curBox(2, 1):curBox(2, 2), ...
            curBox(3, 1):curBox(3, 2)) = curData;
        
        % write to file
        wkwSaveRoi(testDir, curBox(:, 1)', curData);

        %% read data
        curBox = buildRandBox(clen);
        curWkwData = wkwLoadRoi(testDir, curBox, dataType);
        
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

function data = buildRandData(box, className)
    boxSize = 1 + diff(box, 1, 2)';

    switch className
        case {'single', 'double'}
            data = 1 - 2 .* rand(boxSize, className);
        case {'uint8', 'uint16', 'uint32', 'uint64'}
            minMaxVec = minMaxForClass(className);
            data = randi(minMaxVec, boxSize);
        otherwise
            error('Invalid class');
    end
end

function minMax = minMaxForClass(className)
    switch className
        case {'single', 'double'}
            minVal = realmin(className);
            maxVal = realmax(className);
        case {'uint8', 'uint16', 'uint32', 'uint64'}
            minVal = intmin(className);
            maxVal = intmax(className);
        otherwise
            error('Invalid class');
    end
    
    minMax = [minVal, maxVal];
end
