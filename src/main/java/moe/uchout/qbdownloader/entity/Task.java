package moe.uchout.qbdownloader.entity;

import java.io.Serializable;

import lombok.Data;
import lombok.experimental.Accessors;
import moe.uchout.qbdownloader.enums.*;

/**
 * 任务类
 */
@Data
@Accessors(chain = true)
public class Task implements Serializable {
    /**
     * 任务 id
     */
    private int id;

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
    private CurrentTaskStatus CurrentTaskStatus;

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
}
