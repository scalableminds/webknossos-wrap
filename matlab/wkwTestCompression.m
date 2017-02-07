function wkwTestCompression(rawFile, compFile, dataType)
    % wkwTestCompression(rawFile, compFile, dataType)
    %   Loads data for random bounding boxes from a raw and a compressed
    %   WKW file and makes sure that they both contain the same contents.
    % 
    % Written by
    %   Alessandro Motta <alessandro.motta@brain.mpg.de>
    clen = 1024;
    roundCount = 100;
    
    for curIdx = 1:roundCount
        % bounding box
        curBox = nan(3, 2);
        curBox(:, 1) = randi(clen, 3, 1);
        curBox(:, 2) = arrayfun(@(i) ...
            randi([i + 1, clen]), curBox(:, 1));
        
        % load data from raw and compressed files
        rawData = wkwLoadRoi(rawFile, curBox, dataType);
        compData = wkwLoadRoi(compFile, curBox, dataType);
        
        % check for equality
        assert(all(rawData(:) == compData(:)));
    end
end