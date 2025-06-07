package moe.uchout.qbdownloader.enums;

public enum Tags {
    /**
     * 任务标签
     */
    NEW("new_added"),

    /**
     * 总任务完成标签
     */
    TASK_FINISHED_TAG("completed"),

    /**
     * 总任务进行中标签
     */
    TASK_DOWNLOADING_TAG("progressing");

    private final String tag;

    Tags(String tag) {
        this.tag = tag;
    }

    public String getTag() {
        return tag;
    }

    @Override
    public String toString() {
        return tag;
    }
}
