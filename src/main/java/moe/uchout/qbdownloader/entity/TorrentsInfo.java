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
     * 种子下载状态
     */
    private String state;

    /**
     * 种子下载进度
     */
    private float progress;
}
