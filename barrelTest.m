function barrelTest()
    % barrelTest
    %   Does random reads / writes to a barrel file and a
    %   matrix in RAM, and makes sure that the content of
    %   these two are consistent at each point in time.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>

    %% config
    dataType = 'uint8';
    roundCount = 50;
    
    %% preparation
    blockClen = 32;
    fileClen = 1024;
    
    thisDir = fileparts(mfilename('fullpath'));
    fileName = fullfile(thisDir, 'test.dat');
    
    % start empty
    data = zeros(repmat(fileClen, 1, 3), dataType);
    if exist(fileName, 'file'); delete(fileName); end;
    
    % create empty file
    barrelSave(fileName, data, [1, 1, 1]);
    onExit = onCleanup(@() delete(fileName));
    
    % list of possible cube lengths
    clenVec = 2 .^ (log2(blockClen):log2(fileClen));
    
    for curIdx = 1:roundCount
        %% write data
        curWriteClen = clenVec(randi(numel(clenVec)));
        curWriteData = randi( ...
            [intmin(dataType), intmax(dataType)], ...
            repmat(curWriteClen, 1, 3), dataType);
        curWriteOff = 1 + ...
            (randi(fileClen / curWriteClen, 1, 3) - 1) .* curWriteClen;
        
        % update data
        data( ...
            curWriteOff(1):(curWriteOff(1) + curWriteClen - 1), ...
            curWriteOff(2):(curWriteOff(2) + curWriteClen - 1), ...
            curWriteOff(3):(curWriteOff(3) + curWriteClen - 1)) = curWriteData;
        
        % write to file
        barrelSave(fileName, curWriteData, curWriteOff);
        
        %% read data
        curReadClen = clenVec(randi(numel(clenVec)));
        curReadOff = 1 + ...
            (randi(fileClen / curReadClen, 1, 3) - 1) .* curReadClen;
        
        curReadDataFile = barrelLoad( ...
            fileName, curReadClen, curReadOff, dataType);
        curReadData = data( ...
            curReadOff(1):(curReadOff(1) + curReadClen - 1), ...
            curReadOff(2):(curReadOff(2) + curReadClen - 1), ...
            curReadOff(3):(curReadOff(3) + curReadClen - 1));
        
        %% do test
        assert(all(curReadDataFile(:) == curReadData(:)));
        disp(['<< Round ', num2str(curIdx), ' passed']);
    end
end