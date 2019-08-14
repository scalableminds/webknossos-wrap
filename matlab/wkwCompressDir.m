function wkwCompressDir(inRoot, outRoot, taskCount)
    % wkwCompressDir(inRoot, outRoot, taskCount = 10)
    %   Compresses all .wkw files in `inRoot` in parallel (using the
    %   GABA compute cluster, if launched there) and writes the result
    %   to `outRoot`. The output directory must not exist yet.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    wkwFiles = findWkwFiles(inRoot, 2);
    inFiles = fullfile(inRoot, wkwFiles);
    outFiles = fullfile(outRoot, wkwFiles);
    
    % default thread count
    if ~exist('taskCount', 'var') || isempty(taskCount)
        taskCount = 10;
    end
    
    % check output
    if ~exist(fullfile(outRoot, 'header.wkw'), 'file')
        error('Output directories is missing header file');
    end
    
    % make all directories
    outDirs = cellfun(@fileparts, outFiles, 'UniformOutput', false);
    assert(all(cellfun(@mkdir, unique(outDirs))));
    
    % SGE (deprecated)
    % cluster = Cluster.getCluster( ...
    %     '-pe openmp 1', '-l h_vmem=6G', ...
    %     '-l h_rt=0:29:00', ['-tc ', num2str(taskCount)]);

    % SLURM
    cluster = Cluster.config('memory', 6, 'time', '0:29:00', ...
        'taskConcurrency', taskCount);
    jobArgs = cellfun(@(in, out) {{in, out}}, inFiles, outFiles);
    job = Cluster.startJob(@wkwCompress, jobArgs, 'cluster', cluster);
    
    wait(job);
    
    % check that there were no errors
    assert(all(cellfun(@isempty, get(job.Tasks, {'Error'}))));
end

function files = findWkwFiles(inRoot, level)
    isDir = @(e) e.isdir;
    isVis = @(e) not(strncmpi(e.name, '.', 1));
    hasSuffix = @(p, n) strncmpi(fliplr(n), fliplr(p), numel(p));
    isWkw = @(e) hasSuffix('.wkw', e.name);
    
    entries = dir(inRoot);
    dirMask = arrayfun(isDir, entries);
    visMask = arrayfun(isVis, entries);
    wkwMask = arrayfun(isWkw, entries);
    
    if level > 0
        % find dirs
        dirs = entries(dirMask & visMask);
        dirs = {dirs.name};

        % recurse into directories
        recurse = @(d) fullfile( ...
            d, findWkwFiles(fullfile(inRoot, d), level - 1));
        files = cellfun(recurse, dirs, 'UniformOutput', false);
        files = cat(1, files{:});
    else
        % find WKW files
        files = entries(~dirMask & visMask & wkwMask);
        files = {files.name};
        files = files(:);
    end
end
