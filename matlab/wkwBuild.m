function wkwBuild()
    % Written by
    %   Benedikt Staffler <benedikt.staffler@brain.mpg.de>
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    
    buildWithCargo('wkw_load', 'wkwLoadRoi');
    buildWithCargo('wkw_save', 'wkwSaveRoi');
end

function buildWithCargo(oldName, newName)
    prevDir = pwd();
    
    thisDir = fileparts(mfilename('fullpath'));
    cargoDir = fullfile(thisDir, 'rust', oldName);
    
    % build project
    cd(cargoDir);
    system('cargo build --release');
    
    % rename library
    libDir = fullfile(cargoDir, 'target', 'release');
    libPath = fullfile(libDir, strcat('lib', oldName, '.so'));
    mexPath = fullfile(thisDir, strcat(newName, mexext()));
    
    movefile(libPath, mexPath);
    
    cd(prevDir);
end
