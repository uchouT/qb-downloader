package moe.uchout.qbdownloader.entity;

import java.io.Serializable;
import java.util.Map;
import java.util.List;
import java.util.ArrayList;
import java.util.HashMap;

public class TorrentContentNode implements Serializable {
    /**
     * 种子索引，为文件时应该为 filepath 列表的索引；为文件夹时，应该为非索引范围内的不重复内容，无意义
     * 暂定为 1-1 类似的格式
     */
    public String id;

    /**
     * 内容的名称
     */
    public String label;

    /**
     * 子内容 Map
     */
    transient public Map<String, TorrentContentNode> childrenMap;

    /**
     * 子内容 List, 用于传给前端
     */
    public List<TorrentContentNode> children;

    public TorrentContentNode(String id, String label) {
        this.id = id;
        this.label = label;
        this.childrenMap = new HashMap<>();
        this.children = new ArrayList<>();
    }
}
