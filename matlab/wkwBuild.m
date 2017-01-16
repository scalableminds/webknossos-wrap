function wkwBuild()
    % Written by
    %   Benedikt Staffler <benedikt.staffler@brain.mpg.de>
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    prevDir = pwd();
    thisDir = fileparts(mfilename('fullpath'));

    % change to this directory
    cd(thisDir);

    mex -largeArrayDims CXXFLAGS='$CXXFLAGS' ...
        LDOPTIMFLAGS='-O3' CXXOPTIMFLAGS='-O3' ...
        -llz4 -I.. -I../libmorton/include wkwSave.cpp

    mex -largeArrayDims CXXFLAGS='$CXXFLAGS' ...
        LDOPTIMFLAGS='-O3' CXXOPTIMFLAGS='-O3' ...
        -llz4 -I.. -I../libmorton/include wkwLoad.cpp

    mex -largeArrayDims CXXFLAGS='$CXXFLAGS' ...
        LDOPTIMFLAGS='-O3' CXXOPTIMFLAGS='-O3' ...
        -llz4 -I.. -I../libmorton/include wkwCompress.cpp

    % change back
    cd(prevDir);
end
