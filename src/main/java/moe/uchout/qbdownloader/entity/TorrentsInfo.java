package moe.uchout.qbdownloader.entity;

import lombok.Data;
import lombok.experimental.Accessors;

@Data
@Accessors(chain = true)
public class TorrentsInfo {
    /**
     * qb-downloader 接管的种子分类
     */
    static public final String category = "QBD";

    /**
     * 种子 hash
     */
    private String hash;

    /**
     * 种子名称
     */
    private String name;

    /**
     * 种子下载状态
     */
    private String state;

    /**
     * 当前已下载文件大小，单位字节
     */
    private int downloaded;

    /**
     * 种子下载进度
     */
    private float progress;

    /**
     * 种子下载剩余时间
     */
    private String eta;

    /**
     * 种子文件大小
     */
    private int size;
}
