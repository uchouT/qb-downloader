package uchout.qbdownloader.entity;

import lombok.Data;
import lombok.experimental.Accessors;

@Data
@Accessors(chain = true)
public class TorrentContent {
    /**
     * 文件索引
     */
    private int index;

    /**
     * 文件大小
     */
    private int size;
}
