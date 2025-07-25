package moe.uchout.qbdownloader.enums;

public enum Status {
    /**
     * 正在进行间隔任务
     */
    ON_TASK,

    /**
     * 整个分片任务完成，包括间隔任务
     */
    FINISHED,

    /**
     * 分片任务下载完成，需要进行间隔任务
     */
    DOWNLOADED,

    /**
     * 分片任务正在下载
     */
    DOWNLOADING,

    /**
     * 整个任务完成
     */
    ALL_FINISHED,

    /**
     * 任务出错
     */
    ERROR,

    /**
     * 任务暂停中
     */
    PAUSED
}
