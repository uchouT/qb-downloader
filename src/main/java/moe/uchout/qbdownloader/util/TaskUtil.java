package moe.uchout.qbdownloader.util;

import moe.uchout.qbdownloader.enums.*;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import lombok.extern.slf4j.Slf4j;

import java.util.HashMap;
import moe.uchout.qbdownloader.entity.Task;
import moe.uchout.qbdownloader.entity.TorrentContent;
import moe.uchout.qbdownloader.entity.TorrentsInfo;

/**
 * 任务执行相关
 */
@Slf4j
public class TaskUtil extends Thread {

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

    // 添加一个停止标志，使其可以从外部被修改
    private volatile boolean running = true;

    @Override
    public void run() {
        super.setName("qb-downloader-task");
        boolean logined = QbUtil.login();

        while (running) {
            try {
                if (!logined) {
                    logined = QbUtil.login();
                    if (!logined) {
                        Thread.sleep(15000);
                        continue;
                    }
                } else {
                    processTask();
                }
                Thread.sleep(5000);
            } catch (InterruptedException e) {
                running = false;
                Thread.currentThread().interrupt();
                break;
            } catch (Exception e) {
                log.error(e.getMessage());
            }
        }
        // 线程退出前的清理操作
        cleanupBeforeExit();
    }

    /**
     * 处理任务列表中的任务
     */
    private void processTask() {
        try {
            updateTaskStatus();
            for (Task task : TASK_LIST.values()) {
                Status status = task.getStatus();
                if (status == Status.DONWLOADED) {
                    task.runInterval();
                } else if (status == Status.FINISHED) {
                    if (task.getCurrentPieceNum() < task.getTotalPieceNum() - 1) {
                        task.setCurrentPieceNum(task.getCurrentPieceNum() + 1);
                        String hash = task.getHash();
                        QbUtil.delete(hash, true);
                        // 从缓存的种子文件中快速重新添加
                        QbUtil.add(task.getTorrentPath(), true);
                        QbUtil.setNotDownload(task);
                        QbUtil.setPrio(hash, 1, task.getTaskOrder().get(task.getCurrentPieceNum()));
                        QbUtil.removeTag(hash, Tags.NEW);
                        QbUtil.start(hash);
                        task.setStatus(Status.DOWNLOADING);
                    } else {
                        task.setStatus(Status.ALL_FINISHED);
                    }
                }
            }
        } catch (Exception e) {
            // TODO: handle exception
        }
    }

    /**
     * 更新任务的状态
     * 
     * @see Status
     * @see <a href=
     *      "https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-5.0)#get-torrent-list">
     *      qbittorrent API </a>
     */
    private void updateTaskStatus() {
        List<TorrentsInfo> torrentsInfos = QbUtil.getTorrentsInfo();
        for (TorrentsInfo torrentsInfo : torrentsInfos) {
            Task task = TASK_LIST.get(torrentsInfo.getHash());
            if (task.getStatus() == Status.DOWNLOADING) {
                String state = torrentsInfo.getState();
                task.setEta(torrentsInfo.getEta());
                task.setCurrentDownloaded(torrentsInfo.getDownloaded());
                if (state.equals("uploading") ||
                        state.equals("pausedUP") ||
                        state.equals("stalledUP") ||
                        state.equals("queuedUP") ||
                        state.equals("checkingUP") ||
                        state.equals("forceUP") ||
                        state.equals("moving")) {
                    task.setStatus(Status.DONWLOADED);
                }
            }

        }
    }

    /**
     * 线程退出前执行的清理操作
     */
    private void cleanupBeforeExit() {
        sync();
    }

    /**
     * 停止任务线程，优雅退出
     */
    public void stopTask() {

        // 2. 设置停止标志
        running = false;

        // 3. 发送中断信号，确保线程能立即从 sleep 等待状态退出
        this.interrupt();

        // 4. 等待线程结束（可选，视需求而定）
        try {
            this.join(10000); // 最多等待10秒
            if (this.isAlive()) {
                log.error("任务线程未能在规定时间内结束，强制退出");
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
