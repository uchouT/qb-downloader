package moe.uchout.qbdownloader.entity;

import java.io.Serializable;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.LinkedBlockingQueue;

import moe.uchout.qbdownloader.enums.Status;
import cn.hutool.core.thread.ExecutorBuilder;
import lombok.Data;
import lombok.experimental.Accessors;

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
    transient private float totalProcess;

    /**
     * 当前分片任务进度
     */
    transient private float currentProcess;

    /**
     * 任务状态
     */
    private String state;

    /**
     * 分片任务状态
     */
    private Status status;

    /**
     * 内容保存路径
     */
    private String savePath;

    /**
     * 上传路径
     */
    private String uploadPath;

    /**
     * 上传方式, alist 或者 rclone
     */
    private String uploadType;

    /**
     * 分片任务剩余时间
     */
    transient private String eta;

    /**
     * 分片任务总数
     */
    private int totalPieceNum;

    /**
     * 当前进行的分片任务序号, 从 0 开始
     */
    private int currentPieceNum;

    /**
     * 分片任务下载顺序
     */
    private List<List<Integer>> taskOrder;

    /**
     * 文件总数
     */
    private int fileNum;

    /**
     * 
     */
    private List<String> files;

    /**
     * 种子文件备份路径
     */
    private String torrentPath;

    /**
     * 执行间隔任务，标记状态为 ON_TASK, 完成任务后标记为 FINISHED
     */
    public void runInterval() {
        this.status = Status.ON_TASK;
        try {
            // 使用线程池执行上传任务
            EXECUTOR.execute(() -> {
                try {
                    String localPath = this.getSavePath() + "/" + this.getRootDir();
                    String remotePath = this.getUploadPath();

                    // 使用工厂获取上传器并执行上传
                    // TODO: 修改为传入 Task，rclone 和 alist 用不同的方式上传
                    boolean success = moe.uchout.qbdownloader.util.uploader.UploaderFactory
                            .copy(this.getUploadType(), localPath, remotePath);

                    // 根据上传结果设置状态
                    if (success) {
                        this.setStatus(Status.FINISHED);
                    } else {
                        this.setStatus(Status.DONWLOADED); // 上传失败，保持下载完成状态，等待下次尝试
                    }
                } catch (Exception e) {
                    // 发生异常时记录日志并设置状态
                    e.printStackTrace();
                    this.setStatus(Status.DONWLOADED); // 上传发生异常，保持下载完成状态，等待下次尝试
                }
            });
        } catch (Exception e) {
            e.printStackTrace();
            this.status = Status.DONWLOADED; // 提交任务失败，保持下载完成状态
        }
    }
}
