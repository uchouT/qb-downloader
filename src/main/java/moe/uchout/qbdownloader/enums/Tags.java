package moe.uchout.qbdownloader.enums;

public enum Tags {
    /**
     * 添加种子，还没有获取到元数据
     */
    NEW("QBD_new_added"),

    /**
     * 总任务进行中标签
     */
    TASK_PROCESSING_TAG("QBD_progressing"),

    /**
     * 种子已经添加，但是还没有添加到任务中
     */
    WAITED("QBD_waited");

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
