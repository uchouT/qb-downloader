package moe.uchout.qbdownloader.entity;

import moe.uchout.qbdownloader.util.TaskUtil;
import moe.uchout.qbdownloader.util.uploader.UploaderFactory;
import java.io.Serializable;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.LinkedBlockingQueue;
import moe.uchout.qbdownloader.enums.Status;
import cn.hutool.core.thread.ExecutorBuilder;
import lombok.Data;
import lombok.experimental.Accessors;
import lombok.extern.slf4j.Slf4j;

/**
 * 任务类
 */
@Data
@Accessors(chain = true)
@Slf4j
public class Task implements Serializable {
    private static final ExecutorService EXECUTOR = ExecutorBuilder.create()
            .setCorePoolSize(1)
            .setMaxPoolSize(1)
            .setWorkQueue(new LinkedBlockingQueue<>(256))
            .build();
    /**
     * 任务 hash
     */
    private String hash;

    /**
     * 任务名称
     */
    private String name;

    /**
     * 种子根目录
     */
    private String rootDir;

    /**
     * 任务状态
     */
    private Status status;

    /**
     * 内容保存路径
     */
    private String savePath;

    /**
     * 上传路径, 与 uploadType 相对应
     * rclone 为 dest:/path/to/dir
     * alist 为 /path/to/dir
     */
    private String uploadPath;

    /**
     * 上传方式, alist 或者 rclone
     */
    private String uploadType;

    /**
     * 分片任务总数
     */
    private Integer totalPartNum;

    /**
     * 当前进行的分片任务序号, 从 0 开始
     */
    private Integer currentPartNum;

    /**
     * 分片任务下载顺序
     */
    private List<List<Integer>> taskOrder;

    /**
     * 文件总数
     */
    private Integer fileNum;

    /**
     * 种子文件备份路径
     */
    private String torrentPath;

    /**
     * 做种是否还在进行
     */
    private boolean seeding;

    /**
     * 任务最大占用空间
     */
    private long maxSize;

    /**
     * rclone 任务 ID，用于监控状态
     */
    private Integer rcloneJobId;

    // /**
    //  * alist 任务 ID
    //  */
    // private String alistJobId;
    /**
     * 做种时间
     */
    private Integer seedingTimeLimit;

    /**
     * 分享率
     */
    private String ratioLimit;

    /**
     * 执行间隔任务，标记状态为 ON_TASK, 完成间隔任务后标记为 FINISHED
     */
    public void runInterval() {
        this.status = Status.ON_TASK;
        TaskUtil.sync();
        log.info("{} 下载完成，准备上传", this.name);
        try {
            // 使用线程池执行上传任务
            EXECUTOR.execute(() -> {
                // 使用工厂获取上传器并执行上传，非阻塞
                UploaderFactory.copy(this.getUploadType(), this);
            });
        } catch (Exception e) {
            log.error(e.getMessage());
            this.status = Status.ERROR;
        }
    }

    /**
     * 检查间隔任务是否完成
     */
    public void runCheck() {
        EXECUTOR.execute(() -> {
            if (UploaderFactory.check(this.getUploadType(), this)) {
                log.info("上传完成");
                TaskUtil.sync();
                this.status = Status.FINISHED;
            }
        });
    }
}

class TaskConstants {
    /**
     * 上传重试次数
     */
    static final int RETRY = 3;
}
