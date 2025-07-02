package moe.uchout.qbdownloader.util;

import moe.uchout.qbdownloader.enums.*;
import moe.uchout.qbdownloader.exception.QbException;

import java.util.List;
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
        QbUtil.login();
        while (!QbUtil.getLogin() && running) {
            ThreadUtil.sleep(10000);
        }
        log.info("qBittorrent login success.");
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
        cleanupBeforeExit();
    }

    /**
     * 处理任务列表中的任务
     */
    private void processTask() {
        try {
            // TODO: qb 登录后再设置错误账号，可能导致错误
            updateTaskStatus();
            for (Task task : TaskUtil.getTaskList().values()) {
                Status status = task.getStatus();
                if (status == Status.DOWNLOADING || status == Status.PAUSED || status == Status.ALL_FINISHED
                        || status == Status.ERROR) {
                    continue;
                } else if (status == Status.ON_TASK) {
                    task.runCheck();
                } else if (status == Status.DOWNLOADED) {
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
                        // 从保存的种子文件中快速重新添加
                        QbUtil.add(task.getTorrentPath(), task.getSavePath(), task.getSeedingTimeLimit(),
                                task.getRatioLimit());
                        TaskUtil.startTask(currentPartNum + 1, hash, task);
                        TaskUtil.sync();
                        log.info("{} 开始分片任务：{}", task.getName(), task.getCurrentPartNum() + 1);
                    } else {
                        task.setStatus(Status.ALL_FINISHED);
                        TaskUtil.sync();
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
            // 发生在 Qbittorrent 添加，但是任务还没创建完成的时候
            if (task == null) {
                continue;
            }
            if (task.getStatus() == Status.DOWNLOADING) {
                String state = torrentsInfo.getState();
                if (List.of(
                        "uploading",
                        "stalledUP",
                        "queuedUP",
                        "checkingUP",
                        "forcedUP").contains(state)) {
                    task.setSeeding(true);
                    task.setStatus(Status.DOWNLOADED);
                } else if ("stoppedUP".equals(state)) {
                    task.setSeeding(false);
                    task.setStatus(Status.DOWNLOADED);
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

    private void cleanupBeforeExit() {
        TaskUtil.sync();
    }

    public void stopTask() {
        log.info("Stopping task thread...");
        running = false;
        this.interrupt();
        try {
            this.join(5000); // 减少等待时间到5秒
            if (this.isAlive()) {
                log.warn("Task thread did not stop gracefully within 5 seconds, forcing stop...");
                this.join(2000);
                if (this.isAlive()) {
                    log.error("Task thread is still alive after forced stop attempt");
                }
            } else {
                log.info("Task thread stopped gracefully");
            }
        } catch (InterruptedException e) {
            log.warn("Interrupted while waiting for task thread to stop");
            Thread.currentThread().interrupt();
        }
    }
}
