package moe.uchout.qbdownloader.util.uploader;

import moe.uchout.qbdownloader.entity.Task;
import lombok.extern.slf4j.Slf4j;

/**
 * 上传器工厂类
 */
@Slf4j
public class UploaderFactory {
    // 上传类型常量
    public static final String TYPE_RCLONE = "rclone";
    public static final String TYPE_ALIST = "alist";

    private UploaderFactory() {
    }

    /**
     * 根据上传类型获取相应的上传器
     * 
     * @param uploadType 上传类型，"rclone" 或 "alist"
     * @return 对应的上传器，如果类型不支持则返回 null
     */
    public static Uploader getUploader(String uploadType) {
        if (uploadType == null || uploadType.isBlank()) {
            log.warn("上传类型为空，默认使用 Rclone");
            return Rclone.getInstance();
        }

        switch (uploadType.toLowerCase().trim()) {
            case TYPE_RCLONE:
                return Rclone.getInstance();
            case TYPE_ALIST:
                return Alist.getInstance();
            default:
                log.warn("不支持的上传类型: {}，默认使用 Rclone", uploadType);
                return Rclone.getInstance();
        }
    }

    /**
     * 上传文件（静态便捷方法）
     * 
     * @param uploadType 上传类型
     * @param localPath  本地文件路径
     * @param remotePath 远程文件路径
     * @return 是否上传成功
     */
    public static boolean copy(String uploadType, Task task) {
        Uploader uploader = getUploader(uploadType);
        return uploader.copy(task);
    }
}
