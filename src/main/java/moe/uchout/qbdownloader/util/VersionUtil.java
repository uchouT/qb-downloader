package moe.uchout.qbdownloader.util;

public class VersionUtil {
    public static String getVersion() {
        Package pkg = VersionUtil.class.getPackage();
        return (pkg != null) ? pkg.getImplementationVersion() : "UNKNOWN";
    }
}
