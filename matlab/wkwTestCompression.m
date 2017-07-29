function wkwTestCompression()
    % wkwTestCompression
    %   Loads data for random bounding boxes from a raw and a compressed
    %   WKW file and makes sure that they both contain the same contents.
    % 
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    fileLen = 32;
    blockLen = 32;
    dataType = 'uint32';
    clen = fileLen * blockLen;
    
    %% generate dummy data
    thisDir = fileparts(mfilename('fullpath'));
    testRawDir = fullfile(thisDir, 'test_raw');
    testLz4Dir = fullfile(thisDir, 'test_lz4');
    
    wkwInit('new', testRawDir, fileLen, blockLen, dataType, 1);
    rmTestRawDir = onCleanup(@() rmdir(testRawDir, 's'));
    
    wkwInit('compress', testRawDir, testLz4Dir);
    rmTestLz4Dir = onCleanup(@() rmdir(testLz4Dir, 's'));
    
    data = randi( ...
        [intmin(dataType), intmax(dataType)], ...
        repmat(clen, 1, 3), dataType);
    wkwSaveRoi(testRawDir, ones(1, 3), data);
    
    wkwCompress( ...
        fullfile(testRawDir, 'z0', 'y0', 'x0.wkw'), ...
        fullfile(testLz4Dir, 'z0', 'y0', 'x0.wkw'))
    
    %% do the testing
    roundCount = 100;
    
    for curIdx = 1:roundCount
        % bounding box
        curBox = nan(3, 2);
        curBox(:, 1) = randi(clen - 1, 3, 1);
        curBox(:, 2) = arrayfun(@(i) ...
            randi([i + 1, clen]), curBox(:, 1));
        
        % load data from raw and compressed files
        rawData = wkwLoadRoi(testRawDir, curBox);
        compData = wkwLoadRoi(testLz4Dir, curBox);
        
        %% do test
        assert(isequal(rawData, compData));
        disp(['<< Round ', num2str(curIdx), ' passed']);
    end
end