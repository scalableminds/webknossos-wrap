function wkwCompressDir(inRoot, outRoot)
    % wkwCompressDir(inRoot, outRoot)
    %   Compresses all .wkw files in `inRoot` in parallel (using the
    %   GABA compute cluster, if launched there) and writes the result
    %   to `outRoot`. The output directory must not exist yet.
    %
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    wkwFiles = findWkwFiles(inRoot);
    inFiles = fullfile(inRoot, wkwFiles);
    outFiles = fullfile(outRoot, wkwFiles);
    
    % prepare output
    if exist(outRoot, 'dir')
        error('Output directory must not exist');
    end
    
    % make all directories
    outDirs = cellfun(@fileparts, outFiles, 'UniformOutput', false);
    assert(all(cellfun(@mkdir, unique(outDirs))));
    
    cluster = Cluster.getCluster( ...
        '-pe openmp 1', '-l h_vmem=2G', '-l h_rt=0:29:00', '-tc 50');
    jobArgs = cellfun(@(in, out) {{in, out}}, inFiles, outFiles);
    job = Cluster.startJob(@wkwCompress, jobArgs, 'cluster', cluster);
    
    wait(job);
end

function files = findWkwFiles(inRoot)
    isDir = @(e) e.isdir;
    isVis = @(e) not(strncmpi(e.name, '.', 1));
    hasSuffix = @(p, n) strncmpi(fliplr(n), fliplr(p), numel(p));
    isWkw = @(e) hasSuffix('.wkw', e.name);
    
    entries = dir(inRoot);
    dirMask = arrayfun(isDir, entries);
    visMask = arrayfun(isVis, entries);
    wkwMask = arrayfun(isWkw, entries);
    
    % find dirs
    dirs = entries(dirMask & visMask);
    dirs = {dirs.name};
    
    % recurse into directories
    recurse = @(d) fullfile(d, findWkwFiles(fullfile(inRoot, d)));
    dirFiles = cellfun(recurse, dirs, 'UniformOutput', false);
    dirFiles = cat(1, dirFiles{:});
    
    % find WKW files
    files = entries(~dirMask & wkwMask);
    files = {files.name};
    
    % build output
    files = cat(1, files(:), dirFiles);
end