package moe.uchout.qbdownloader.util;

import moe.uchout.qbdownloader.enums.*;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.io.File;
import com.google.gson.reflect.TypeToken;

import cn.hutool.core.lang.Assert;
import cn.hutool.core.thread.ThreadUtil;

import java.lang.reflect.Type;
import java.io.InputStreamReader;
import java.io.FileOutputStream;
import java.io.OutputStreamWriter;
import java.io.FileInputStream;
import lombok.extern.slf4j.Slf4j;

import java.util.HashMap;
import moe.uchout.qbdownloader.entity.Task;
import moe.uchout.qbdownloader.entity.TorrentContent;

/**
 * 任务执行相关
 */
@Slf4j
public class TaskUtil {

    private static final String TASK_FILE_PATH = "configs/tasks.json";
    private static final String TORRENT_FILE_PATH = "configs/torrents/";
    /**
     * 任务列表
     */
    private static final Map<String, Task> TASK_LIST = new HashMap<>();

    public static Map<String, Task> getTaskList() {
        return TASK_LIST;
    }

    /**
     * 添加任务 TODO: 仅提供测试，具体添加任务要在前端交互
     */
    public static void addTask(String url, String uploadType, String savePath, String uploadPath, int maxSize) {
        try {
            QbUtil.add(url, false);
            ThreadUtil.sleep(1000);
            String hash = QbUtil.getHash();
            QbUtil.export(hash, TORRENT_FILE_PATH + hash + ".torrent");
            String name = QbUtil.getName(hash);
            QbUtil.removeTag(hash, Tags.NEW);
            Task task = new Task().setCurrentPartNum(0).setStatus(Status.PAUSED).setName(name)
                    .setHash(hash).setSeeding(false).setTorrentPath(TORRENT_FILE_PATH + hash + ".torrent")
                    // 以下内容由用户设置
                    .setUploadType(uploadType)
                    .setSavePath(savePath) // savePath 需要去除末尾 /
                    .setUploadPath(uploadPath)
                    .setMaxSize(maxSize * 1024 * 1024);
            List<TorrentContent> contents = QbUtil.getTorrentContentList(hash, task);
            List<List<Integer>> order = getTaskOrder(contents, task.getMaxSize());
            task.setTotalPartNum(order.size());
            task.setTaskOrder(order);
            TASK_LIST.put(hash, task);
            sync();
            Thread.sleep(1000);
            boolean setNotDownload = QbUtil.setNotDownload(task);
            Assert.isTrue(setNotDownload, "设置不下载失败");
            QbUtil.setPrio(hash, 1, task.getTaskOrder().get(0));
            QbUtil.start(hash);
            task.setStatus(Status.DOWNLOADING);
            log.info("添加任务成功: {}", task.getName());
        } catch (Exception e) {
            log.error("添加任务失败: {}", e.getMessage(), e);
            throw new RuntimeException(e);
        }
    }

    /**
     * 从文件获取任务列表
     */
    public static synchronized void load() {
        File taskFile = new File(TASK_FILE_PATH);
        if (!taskFile.exists()) {
            log.debug("任务文件不存在，无需加载");
            return;
        }
        try (InputStreamReader reader = new InputStreamReader(
                new FileInputStream(taskFile), "UTF-8")) {
            Type type = new TypeToken<Map<String, Task>>() {
            }.getType();
            Map<String, Task> loaded = GsonStatic.gson.fromJson(reader, type);
            if (loaded != null) {
                TASK_LIST.clear();
                TASK_LIST.putAll(loaded);
                log.info("加载任务列表成功，共 {} 个任务", TASK_LIST.size());
            }
        } catch (Exception e) {
            log.error("加载任务列表失败", e);
        }
    }

    /**
     * 同步任务列表, 将任务保存到文件中
     */
    public static synchronized void sync() {
        if (TASK_LIST.isEmpty()) {
            log.debug("任务列表为空，无需保存");
            return;
        }
        try (OutputStreamWriter writer = new OutputStreamWriter(
                new FileOutputStream(TASK_FILE_PATH), "UTF-8")) {
            String json = GsonStatic.toJson(TASK_LIST);
            writer.write(json);
            log.debug("任务列表已保存，共 {} 个任务", TASK_LIST.size());
        } catch (Exception e) {
            log.error("保存任务列表失败", e);
        }
    }

    /**
     * 删除任务，根据 hash
     * 
     * @param hash
     */
    public static void delete(String hash) {
        TASK_LIST.remove(hash);
        sync();
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
        List<Integer> onePart = new ArrayList<>();
        int currentSize = 0;
        for (int i = 0, size = torrentContentList.size(); i < size; i++) {
            TorrentContent torrentContent = torrentContentList.get(i);
            if (currentSize + torrentContent.getSize() > maxSize) {
                TaskOrder.add(onePart);
                onePart = new ArrayList<>();
                currentSize = 0;
            }
            onePart.add(torrentContent.getIndex());
            if (i == size - 1) {
                TaskOrder.add(onePart);
            }
            currentSize += torrentContent.getSize();
        }
        return TaskOrder;
    }
}
