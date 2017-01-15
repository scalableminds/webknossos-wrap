function barrelFromKnossos(wkParam, brlParam, box)
    % barrelFromKnossos(wkParam, box, outDir)
    %   Converts a KNOSSOS hierarchy into barrel files.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    % config
    fileClen = 1024;
    
    % align box with barrel files
    fileIds = [ ...
        floor((box(:, 1) - 1) ./ fileClen), ...
        ceil(box(:, 2) ./ fileClen) - 1];
    
    % progress
    tic;
    curCount = 1;
    fileCount = prod(diff(fileIds, 1, 2) + 1);
    
    % copy files
    for curIdxX = fileIds(1, 1):fileIds(1, 2)
        for curIdxY = fileIds(2, 1):fileIds(2, 2)
            for curIdxZ = fileIds(3, 1):fileIds(3, 2)
                % show progress
                Util.progressBar(curCount, fileCount);
                curCount = curCount + 1;
                
                % box
                curIds = [ ...
                    curIdxX, curIdxY, curIdxZ];
                curBox = [ ...
                    max(box(:, 1),  1 + curIds(:)  .* fileClen), ...
                    min(box(:, 2), (1 + curIds(:)) .* fileClen)];
                curOffset = curBox(:, 1)';
                
                % do the work
                curData = readKnossosRoi( ...
                    wkParam.root, wkParam.prefix, curBox);
                barrelSaveRoi( ...
                    brlParam.root, brlParam.prefix, curOffset, curData);
            end
        end
    end
end