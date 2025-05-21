package moe.uchout.qbdownloader.enums;

public enum Status {
    /**
     * 正在进行间隔任务
     */
    ON_TASK,

    /**
     * 整个分片任务完成
     */
    FINISHED,

    /**
     * 分片任务下载完成，需要进行间隔任务
     */
    DONWLOADED,

    /**
     * 分片任务正在下载
     */
    DOWNLOADING
}
