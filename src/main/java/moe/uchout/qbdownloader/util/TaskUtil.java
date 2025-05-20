package moe.uchout.qbdownloader.util;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.HashMap;

import moe.uchout.qbdownloader.entity.Task;
import moe.uchout.qbdownloader.entity.TorrentContent;

/**
 * 任务执行相关
 */
public class TaskUtil {

    /**
     * 任务列表
     */
    private static final Map<String, Task> TASK_LIST = new HashMap<>();

    /** 添加任务 */
    public static void addTask(Task task) {
        TASK_LIST.put(task.getHash(), task);
        sync();
    }

    /**
     * 从文件获取任务列表
     */
    public static synchronized void load() {

    }

    /**
     * 同步任务列表, 将任务保存到文件中
     */
    public static synchronized void sync() {

    }

    /**
     * 删除任务，根据 hash
     * 
     * @param hash
     */
    public static void delete(String hash) {

    }

    /**
     * 检测任务是否可在最大分片大小范围内完成
     * 
     * @param torrentContents
     * @param maxSize
     * @return 是否可以完成
     */
    public static boolean check(List<TorrentContent> torrentContents, int maxSize) {
        for (TorrentContent torrentContent : torrentContents) {
            if (torrentContent.getSize() > maxSize) {
                return false;
            }
        }
        return true;
    }

    /**
     * 分片任务下载顺序
     * 
     * @param torrentContentList 种子内容列表
     * @param maxSize            最大分片大小
     * @return 二维数组，每个元素是 index 列表
     */
    public static List<List<Integer>> getTaskOrder(List<TorrentContent> torrentContentList, int maxSize)
            throws Exception {
        if (!check(torrentContentList, maxSize)) {
            throw new IllegalArgumentException("任务过大，无法分片");
        }
        List<List<Integer>> TaskOrder = new ArrayList<>();
        List<Integer> onePiece = new ArrayList<>();
        int currentSize = 0;
        for (TorrentContent torrentContent : torrentContentList) {
            if (currentSize + torrentContent.getSize() > maxSize) {
                TaskOrder.add(onePiece);
                onePiece = new ArrayList<>();
                currentSize = 0;
            }
            onePiece.add(torrentContent.getIndex());
            currentSize += torrentContent.getSize();
        }
        return TaskOrder;
    }
}
