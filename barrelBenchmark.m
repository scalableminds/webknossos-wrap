function barrelBenchmark(param)    
    [barrelFile, wkDir, cls, sz, testFilename1, testFilename2] = buildData(param);
    barrelCompFile = strcat(barrelFile, '.lz4');
    
    sizeVec = 2 .^ (0:log2(sz));
    sizeCount = numel(sizeVec);
    repCount = 20;
    
    disp('<< Running benchmark');
    data = nan(sizeCount, 5, repCount);
    
    for curSizeIdx = 1:sizeCount
        curSize = sizeVec(curSizeIdx);
        
        for curRep = 1:repCount
            % randomly select a cube
            curCubeCount = sz / curSize - 1;
            curCube = randi(uint64([0, curCubeCount]), 1, 3);
            curRoi = [ ...
                 1 + curCube(:)  .* curSize, ...
                (1 + curCube(:)) .* curSize];
            
            % KNOSSOS
            tic;
            readKnossosRoi(wkDir, 'bench', curRoi, cls);
            data(curSizeIdx, 1, curRep) = toc;
            
            % barrel (raw)
            tic;
            barrelLoad(barrelFile, curSize, curRoi(:, 1)', cls);
            data(curSizeIdx, 2, curRep) = toc;
            
            % barrel (compressed)
            if curSize >= 32
                tic;
                barrelLoad(barrelCompFile, curSize, curRoi(:, 1)', cls);
                data(curSizeIdx, 3, curRep) = toc;
            end
            
            % hdf5 (uncompressed)
            tic;
            h5read(testFilename1,'/seg', curRoi(:, 1)', repmat(curSize,3,1));
            data(curSizeIdx, 4, curRep) = toc;
            
            % hdf5 (compressed)
            tic;
            h5read(testFilename2,'/seg', curRoi(:, 1)', repmat(curSize,3,1));
            data(curSizeIdx, 5, curRep) = toc;
           
        end
    end
    
    % convert to throughput
    data = bsxfun(@times, sizeVec(:) .^ 3, 1 ./ data);
    meanMat = mean(data, 3); stdMat = std(data, 0, 3);
    
    %%
    disp('<< Plotting');
    
    figure; hold('on'); grid('on');
    errorbar(log2(sizeVec), meanMat(:, 1), stdMat(:, 1));
    errorbar(log2(sizeVec), meanMat(:, 2), stdMat(:, 2));
    errorbar(log2(sizeVec), meanMat(:, 3), stdMat(:, 3));
    errorbar(log2(sizeVec), meanMat(:, 4), stdMat(:, 4));
    errorbar(log2(sizeVec), meanMat(:, 5), stdMat(:, 5));
    % X axis
    xticks(log2(sizeVec));
    xticklabels(arrayfun(@num2str, sizeVec, 'UniformOutput', false));
    xlabel('Side length of loaded cube [voxel]');
    
    % Y axis
    ylabel('Throughput [voxel / s]');
    legend( ...
        'KNOSSOS (128 voxels)', ...
        'Barrel (1024 voxels, raw)', ...
        'Barrel (1024 voxels, LZ4-HC compressed)', ...
        'HDF5 (1024 voxels, uncompressed)', ...
        'HDF5 (1024 voxels, deflate compressed)', ...
        'Location', 'NorthWest');
end

function [barrelFile, wkDir, cls, sz, testFilename1, testFilename2] = buildData(param)
    cls = 'uint32';
    sz = 1024;
    
    % load data
    box = [5121, 3584, 1728];
    box = [box(:) - sz / 2, box(:) + sz / 2 - 1];
    data = loadSegDataGlobal(param.seg, box);
    
    % create folder
    benchDir = fullfile(pwd, 'benchmark');
    if exist(benchDir, 'dir'); rmdir(benchDir, 's'); end;
    mkdir(benchDir);
    
    % save barrel encoded data
    disp('<< Writing raw barrel file...');
    barrelFile = fullfile(benchDir, 'barrel.raw');
    barrelSave(barrelFile, data, [1, 1, 1]);
    
    disp('<< Compressing barrel file...');
    barrelCompress(barrelFile, strcat(barrelFile, '.lz4'));
    
    % save KNOSSOS data
    wkDir = fullfile(benchDir, 'knossos'); mkdir(wkDir);
    writeKnossosRoi(wkDir, 'bench', [1, 1, 1], data, cls);
    
    % save as HDF5 file 
    testFilename1 = fullfile(benchDir, 'data.hdf5');
    testFilename2 = fullfile(benchDir, 'data_c.hdf5');
    h5create(testFilename1, '/seg', size(data), 'Datatype', 'uint32', 'ChunkSize', [32 32 32], 'Deflate', 0);
    h5write(testFilename1, '/seg', data);
    h5create(testFilename2, '/seg', size(data), 'Datatype', 'uint32', 'ChunkSize', [32 32 32], 'Deflate', 1);
    h5write(testFilename2, '/seg', data);
end
