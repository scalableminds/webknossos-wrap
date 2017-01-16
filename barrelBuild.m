function barrelBuild()
    % Written by
    %   Benedikt Staffler <benedikt.staffler@brain.mpg.de>
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    prevDir = pwd();
    thisDir = fileparts(mfilename('fullpath'));
    
    % change to this directory
    cd(thisDir);
    
    mex -largeArrayDims CXXFLAGS='$CXXFLAGS' ...
        LDOPTIMFLAGS='-O3' CXXOPTIMFLAGS='-O3' ...
        -llz4 -I. -Ilibmorton/include barrelSave.cpp
    
    mex -largeArrayDims CXXFLAGS='$CXXFLAGS' ...
        LDOPTIMFLAGS='-O3' CXXOPTIMFLAGS='-O3' ...
        -llz4 -I. -Ilibmorton/include barrelLoad.cpp
    
    mex -largeArrayDims CXXFLAGS='$CXXFLAGS' ...
        LDOPTIMFLAGS='-O3' CXXOPTIMFLAGS='-O3' ...
        -llz4 -I. -Ilibmorton/include barrelCompress.cpp
    
    % change back
    cd(prevDir);
end