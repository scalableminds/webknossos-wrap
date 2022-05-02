function wkwBuild()
    % Written by
    %   Benedikt Staffler <benedikt.staffler@brain.mpg.de>
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>

    % export path to library
    matlabRoot = matlabroot();
    
    % path to .so / .dll files
    arch = computer('arch');
    extraLinkPaths = {fullfile(matlabRoot, 'bin', arch)};
    
    if ispc
        % on Windows, we also require
        % - the corresponding .lib files
        extraLinkPaths{end + 1} = ...
            fullfile(matlabRoot, 'extern', 'lib', arch, 'microsoft');
    end
    
    % make link paths available for cargo
    exportExtraLinkPaths(extraLinkPaths);
    
    buildWithCargo('wkw_compress', 'wkwCompress');
    buildWithCargo('wkw_init', 'wkwInit');
    buildWithCargo('wkw_load', 'wkwLoadRoi');
    buildWithCargo('wkw_save', 'wkwSaveRoi');
end

function buildWithCargo(oldName, newName)
    prevDir = pwd();
    prevRestore = onCleanup(@() cd(prevDir));
    
    thisDir = fileparts(mfilename('fullpath'));
    cargoDir = fullfile(thisDir, 'rust', oldName);
    
    % build project
    cd(cargoDir);
    system('cargo clean');
    system('cargo update');
    
    if ismac
        % In case the binary is build on arm64 make sure to use x86 as the
        % target.
        system('cargo build --release --target=x86_64-apple-darwin');
        libDir = fullfile(cargoDir, 'target', 'x86_64-apple-darwin', 'release');
    else
        system('cargo build --release');
        libDir = fullfile(cargoDir, 'target', 'release');
    end
    
    % rename library
    if ismac
        libPath = fullfile(libDir, strcat('lib', oldName, '.dylib'));
    elseif isunix
        libPath = fullfile(libDir, strcat('lib', oldName, '.so'));
    elseif ispc
        libPath = fullfile(libDir, strcat(oldName, '.dll'));
    else
        error('Platform not supported');
    end
    
    mexPath = fullfile(thisDir, strcat(newName, '.', mexext()));
    copyfile(libPath, mexPath);
    
    cd(prevDir);
end

function exportExtraLinkPaths(paths)
    extraLinkPathsStr = strjoin(paths, ';');
    setenv('EXTRALINKPATHS', extraLinkPathsStr);
end
