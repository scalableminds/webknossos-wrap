function mortonSetup()
    % Written by
    %   Benedikt Staffler <benedikt.staffler@brain.mpg.de>
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    prevDir = pwd();
    thisDir = fileparts(mfilename('fullpath'));
    
    % change to this directory
    cd(thisDir);

    mex -largeArrayDims CXXFLAGS='$CXXFLAGS' ...
        LDOPTIMFLAGS='-O3' CXXOPTIMFLAGS='-O3 -DNDEBUG' ...
        -Ilibmorton/include mortonLoad.cpp
    mex -largeArrayDims CXXFLAGS='$CXXFLAGS' ...
        LDOPTIMFLAGS='-O3' CXXOPTIMFLAGS='-O3 -DNDEBUG' ...
        -Ilibmorton/include mortonSave.cpp
    
    % change back
    cd(prevDir);
end