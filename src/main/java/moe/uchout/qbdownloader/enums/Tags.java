package moe.uchout.qbdownloader.enums;

public enum Tags {
    /**
     * 任务标签
     */
    NEW("new_added"),

    /**
     * 任务完成标签
     */
    TASK_FINISHED_TAG("task_finished_tag"),

    /**
     * 任务下载完成标签
     */
    TASK_DOWNLOADED_TAG("task_downloaded_tag"),

    /**
     * 任务正在下载标签
     */
    TASK_DOWNLOADING_TAG("task_downloading_tag");

    private final String tag;

    Tags(String tag) {
        this.tag = tag;
    }

    public String getTag() {
        return tag;
    }
}
