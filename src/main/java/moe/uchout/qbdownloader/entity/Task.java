package moe.uchout.qbdownloader.entity;

import java.io.Serializable;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.LinkedBlockingQueue;

import cn.hutool.core.thread.ExecutorBuilder;
import lombok.Data;
import lombok.experimental.Accessors;
import moe.uchout.qbdownloader.enums.*;

/**
 * 任务类
 */
@Data
@Accessors(chain = true)
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
     * 总任务进度
     */
    private float totalProcess;

    /**
     * 当前分片任务进度
     */
    private float currentProcess;

    /**
     * 总任务状态
     */
    private TaskStatus status;

    /**
     * 当前分片任务状态
     */
    private CurrentTaskStatus currentTaskStatus;

    /**
     * 内容保存路径
     */
    private String savePath;

    /**
     * rclone 上传路径
     */
    private String uploadPath;

    /**
     * 分片任务剩余时间
     */
    private String eta;

    /**
     * 分片任务总数
     */
    private int totalPieceNum;

    /**
     * 当前进行的分片任务序号, 从 1 开始
     */
    private int currentPieceNum;

    /**
     * 分片任务下载顺序
     */
    private List<List<Integer>> taskOrder;

    public static void runInterval() {
        // TODO
        EXECUTOR.execute();
    }
}
