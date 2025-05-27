package moe.uchout.qbdownloader.util;

import moe.uchout.qbdownloader.enums.*;
import moe.uchout.qbdownloader.exception.QbException;

import java.util.List;
import cn.hutool.core.lang.Assert;
import cn.hutool.core.thread.ThreadUtil;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.entity.Task;
import moe.uchout.qbdownloader.entity.TorrentsInfo;

@Slf4j
public class TaskThread extends Thread {
    // 添加一个停止标志，使其可以从外部被修改
    private volatile boolean running = true;

    public TaskThread() {
        super.setName("qb-downloader-task");
    }

    @Override
    public void run() {
        try {
            QbUtil.login();
        } catch (QbException e) {
            log.warn(e.getMessage());
            ThreadUtil.sleep(10000);
        }
        while (running) {
            try {
                processTask();
                Thread.sleep(5000);
            } catch (InterruptedException e) {
                running = false;
                Thread.currentThread().interrupt();
                break;
            } catch (Exception e) {
                log.error("任务处理出错", e);
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
            for (Task task : TaskUtil.getTaskList().values()) {
                Status status = task.getStatus();
                if (status == Status.DOWNLOADING || status == Status.PAUSED || status == Status.ALL_FINISHED) {
                    continue;
                } else if (status == Status.ON_TASK) {
                    task.runCheck();
                } else if (status == Status.DONWLOADED) {
                    task.runInterval();
                    log.info("运行间隔任务");
                } else if (status == Status.FINISHED) {
                    if (task.isSeeding()) {
                        continue;
                    }
                    int currentPartNum = task.getCurrentPartNum();
                    String hash = task.getHash();
                    QbUtil.delete(hash, true);
                    if (currentPartNum < task.getTotalPartNum() - 1) {
                        task.setCurrentPartNum(currentPartNum + 1);
                        // 从缓存的种子文件中快速重新添加
                        QbUtil.add(task.getTorrentPath(), true);
                        Thread.sleep(1000);
                        boolean setNotDownload = QbUtil.setNotDownload(task);
                        for (int i = 0; !setNotDownload && i < TaskConstants.RETRY_TIMES; i++) {
                            setNotDownload = QbUtil.setNotDownload(task);
                        }
                        Assert.isTrue(setNotDownload, "设置不下载失败");

                        QbUtil.setPrio(hash, 1, task.getTaskOrder().get(currentPartNum + 1));
                        QbUtil.start(hash);
                        task.setStatus(Status.DOWNLOADING);
                        log.info("{} 开始分片任务：{}", task.getName(), task.getCurrentPartNum() + 1);
                    } else {
                        task.setStatus(Status.ALL_FINISHED);
                        log.info("任务: {} 全部完成", task.getName());
                    }
                }
            }
        } catch (Exception e) {
            log.error("任务处理异常", e);
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
            Task task = TaskUtil.getTaskList().get(torrentsInfo.getHash());
            if (task.getStatus() == Status.DOWNLOADING) {
                String state = torrentsInfo.getState();
                if (List.of(
                        "uploading",
                        "stalledUP",
                        "queuedUP",
                        "checkingUP",
                        "forcedUP").contains(state)) {
                    task.setSeeding(true);
                    task.setStatus(Status.DONWLOADED);
                } else if ("stoppedUP".equals(state)) {
                    task.setSeeding(false);
                    task.setStatus(Status.DONWLOADED);
                } else if (List.of(
                        "error",
                        "missingFiles").contains(state)) {
                    task.setStatus(Status.ERROR);
                }
            } else if (task.isSeeding()) {
                if ("stoppedUP".equals(torrentsInfo.getState())) {
                    task.setSeeding(false);
                }
            }
        }
    }

    /**
     * 线程退出前执行的清理操作 TODO
     */
    private void cleanupBeforeExit() {
        TaskUtil.sync();
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

class TaskConstants {
    static final int RETRY_TIMES = 3;
}