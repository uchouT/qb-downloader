package moe.uchout.qbdownloader.util.uploader;

/**
 * 上传器接口
 */
public interface Uploader {
    /**
     * 上传文件到远程存储
     * 
     * @param localPath  本地文件路径
     * @param remotePath 远程存储路径
     * @return 上传是否成功
     */
    boolean copy(String localPath, String remotePath);
    
    /**
     * 检查上传器状态
     * 
     * @return 上传器是否可用
     */
    boolean check();
}
