function path = wkwBuildFilePath(cubeIds)
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    assert(all(cubeIds >= 0));
    assert(all(mod(cubeIds, 1) == 0));
    
    path = fullfile( ...
        strcat('z', sprintf('%u', cubeIds(3))), ...
        strcat('y', sprintf('%u', cubeIds(2))), ...
        strcat('x', sprintf('%u', cubeIds(1)), '.wkw'));
end