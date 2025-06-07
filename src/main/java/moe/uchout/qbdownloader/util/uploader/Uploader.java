package moe.uchout.qbdownloader.util.uploader;

import moe.uchout.qbdownloader.entity.Task;

/**
 * 上传器接口
 */
public interface Uploader {
    /**
     * 上传文件到远程存储
     * 
     * @param task 待上传的任务
     * @return 上传是否成功
     */
    void copy(Task task);

    /**
     * 检查上传器状态
     */
    boolean check(Task task);
}
