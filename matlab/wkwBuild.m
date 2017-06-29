function wkwBuild()
    % Written by
    %   Benedikt Staffler <benedikt.staffler@brain.mpg.de>
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>

    % export path to library
    matlabRoot = matlabroot();
    mexLibRoot = fullfile(matlabRoot, 'bin', 'glnxa64');
    setenv('MEXLIBROOT', mexLibRoot);
    
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
    system('cargo update');
    system('cargo build --release');
    
    % rename library
    libDir = fullfile(cargoDir, 'target', 'release');
    libPath = fullfile(libDir, strcat('lib', oldName, '.so'));
    mexPath = fullfile(thisDir, strcat(newName, '.', mexext()));
    
    movefile(libPath, mexPath);
    
    cd(prevDir);
end
