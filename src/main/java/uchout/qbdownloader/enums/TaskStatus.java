package uchout.qbdownloader.enums;

/**
 * 分片任务的状态
 */
public enum TaskStatus {
    /**
     * qBittorrent 下载完了当前分片，并且间隔任何完成
     */
    COMPLETED,

    /**
     * qBittorrent 正在下载当前分片
     */
    DOWNLOADING,

    /**
     * qBittorrent 正在进行间隔任务，已将所有要执行的间隔任务发起
     */
    ON_TASK,

    /**
     * 分片任务还未开始
     */
    NOT_STARTED,

    /**
     * qBittorrent 下载完成，但是还没有执行间隔任务
     */
    DOWNLOADED
}
