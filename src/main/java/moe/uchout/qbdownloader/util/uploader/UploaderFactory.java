package moe.uchout.qbdownloader.util.uploader;

import moe.uchout.qbdownloader.entity.Task;
import cn.hutool.core.util.StrUtil;
import lombok.extern.slf4j.Slf4j;

/**
 * 上传器工厂类
 */
@Slf4j
public class UploaderFactory {
    // 上传类型常量
    public static final String TYPE_RCLONE = "rclone";
    // public static final String TYPE_ALIST = "alist";

    private UploaderFactory() {
    }

    /**
     * 根据上传类型获取相应的上传器
     * 
     * @param uploadType 上传类型，"rclone" 或 "alist"
     * @return 对应的上传器，如果类型不支持则返回 null
     */
    public static Uploader getUploader(String uploadType) {
        if (StrUtil.isBlank(uploadType)) {
            log.warn("上传类型为空，默认使用 Rclone");
            return Rclone.getInstance();
        }

        switch (uploadType.toLowerCase().trim()) {
            case TYPE_RCLONE:
                return Rclone.getInstance();
            // case TYPE_ALIST:
            // return Alist.getInstance();
            default:
                log.warn("不支持的上传类型: {}，默认使用 Rclone", uploadType);
                return Rclone.getInstance();
        }
    }

    /**
     * 上传文件到远程存储
     * 
     * @param uploadType
     * @param task
     */
    public static void copy(String uploadType, Task task) {
        Uploader uploader = getUploader(uploadType);
        uploader.copy(task);
    }

    /**
     * 检查上传器状态
     * 
     * @param uploadType 上传类型
     * @param task       任务对象
     */
    public static boolean check(String uploadType, Task task) {
        Uploader uploader = getUploader(uploadType);
        return uploader.check(task);
    }
}
