function wkwTestLatency(wkwRoot, box, dataType)
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    % config
    blen = 32;
    clen = 1024;
    
    box(:, 2) = box(:, 2) + 1;
    boxBlocks = ceil(box ./ blen);
    
    % prepare
    data = [];
    
    while true
        % define block
        curBlockIds = arrayfun(@(idx) randi(boxBlocks(idx, :)), 1:3);
        curBlockIds = reshape(curBlockIds, [], 1);
        
        curCubeIds = floor(curBlockIds ./ (clen / blen));
        curBoxOff = curCubeIds * clen;
        
        curBox = curBlockIds * blen;
        curBox = bsxfun(@minus, curBox, curBoxOff);
        curOff = curBox(:, 1) + 1; 
        
        % build path
        wkwPath = wkwBuildFilePath(curCubeIds);
        wkwPath = fullfile(wkwRoot, wkwPath);
        
        tic();
        wkwLoad(wkwPath, blen, curOff, dataType);
        curData = toc();
        
        % evaluate data
        data(end + 1) = 1E3 * curData; %#ok
        
        dataMean = mean(data);
        dataStd = std(data);
        dataPrct = prctile(data, [5, 50, 95]);
        
        fprintf('Round %d:\n', numel(data));
        fprintf('  %.2f ± %.2f (mean ± std) ms\n', dataMean, dataStd);
        fprintf( ...
            '  %.2f - %.2f - %.2f (5th - 50th - 95th prct.) ms\n', ...
            dataPrct(1), dataPrct(2), dataPrct(3));
        
        % wait for a second
        pause(1);
    end
end