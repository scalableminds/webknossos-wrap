import java.nio.ByteBuffer;

public class BarrelTest {
    static { System.load("/u/amotta/temp/auxiliaryMethods/mex/barrel/java/BarrelTest.so"); }

    public static native int readRawData(String file, long x, long y, long z, int clen, ByteBuffer buf);
    public static void main(String[] args){
        String dir = "/gaba/u/mberning/wkCubes/2012-09-28_ex145_07x2_ROI2016_corrected_brlFormat/color/1/";
        String fileName = "2012-09-28_ex145_07x2_ROI2016_corrected_brlFormat_mag1_x000000_y000000_z000000.brl";
        String path = dir + fileName;

        ByteBuffer buf = ByteBuffer.allocateDirect(32 * 32 * 32);
        int err = readRawData(path, 128, 0, 512, 32, buf);

        if(err != 0){
            System.err.println("readRawData failed with exit code " + err);
        }
    }
}
