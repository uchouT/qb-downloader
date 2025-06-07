package moe.uchout.qbdownloader.util;

import moe.uchout.qbdownloader.enums.*;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.io.File;
import com.google.gson.reflect.TypeToken;

import cn.hutool.core.io.FileUtil;
import cn.hutool.core.lang.Assert;
import cn.hutool.core.thread.ThreadUtil;

import java.lang.reflect.Type;
import java.io.InputStreamReader;
import java.io.FileOutputStream;
import java.io.OutputStreamWriter;
import java.io.FileInputStream;

import lombok.Synchronized;
import lombok.extern.slf4j.Slf4j;

import java.util.HashMap;
import java.util.LinkedHashMap;

import moe.uchout.qbdownloader.api.entity.TorrentRes;
import moe.uchout.qbdownloader.entity.Task;
import com.dampcake.bencode.Bencode;

/**
 * 任务执行相关
 */
@Slf4j
public class TaskUtil {
    private static final String TASK_FILE_PATH = ConfigUtil.CONFIG_DIR + File.separator + "task.json";
    private static final String TORRENT_FILE_PATH = ConfigUtil.CONFIG_DIR + File.separator + "torrents"
            + File.separator;
    /**
     * 任务列表
     */
    private static final Map<String, Task> TASK_LIST = new HashMap<>();

    public static Map<String, Task> getTaskList() {
        return TASK_LIST;
    }

    public static void start(String hash) {
        Task task = TASK_LIST.get(hash);
        if (task == null) {
            throw new IllegalArgumentException("Task not found: " + hash);
        }
        try {
            QbUtil.start(hash);
            task.setStatus(Status.DOWNLOADING);
            sync();
            log.info("任务开始: {}", task.getName());
        } catch (Exception e) {
            log.error("启动任务失败: {}", e.getMessage(), e);
            throw new RuntimeException("Failed to start task: " + e.getMessage(), e);
        }
    }

    public static void stop(String hash) {
        Task task = TASK_LIST.get(hash);
        if (task == null) {
            throw new IllegalArgumentException("Task not found: " + hash);
        }
        try {
            QbUtil.pause(hash);
            task.setStatus(Status.PAUSED);
            sync();
            log.info("任务暂停: {}", task.getName());
        } catch (Exception e) {
            log.error("暂停任务失败: {}", e.getMessage(), e);
            throw new RuntimeException("Failed to stop task: " + e.getMessage(), e);
        }
    }

    /**
     * 添加种子任务，不下载，同时将元数据保存到指定文件夹
     * 
     * @param isFile
     * @param file
     * @param url
     * @param savePath
     * @return 种子 hash 值
     */
    public static String addTorrent(boolean isFile, byte[] file, String url, String savePath) {
        if (isFile) {
            QbUtil.add(file, url, savePath);
        } else {
            QbUtil.add(url, savePath, false);
        }
        ThreadUtil.sleep(500);
        String hash = QbUtil.getHash();
        QbUtil.export(hash, TORRENT_FILE_PATH + hash + ".torrent");
        return hash;
    }

    /**
     * TODO: 添加前查重，添加下载内容可选
     * 根据种子的响应添加任务
     * 
     * @param torrentRes       种子响应对象
     * @param uploadType       上传类型 "rclone" or "alist"
     * @param uploadPath       上传路径
     * @param maxSize          最大分片大小，单位为 MB
     * @param seedingTimeLimit
     * @param ratioLimit
     */
    @Synchronized("TASK_LIST")
    public static void addTask(TorrentRes torrentRes, String uploadType,
            String uploadPath, long maxSize, int seedingTimeLimit, String ratioLimit) {
        try {
            String savePath = torrentRes.getSavePath();
            String hash = torrentRes.getHash();
            String name = QbUtil.getName(hash);
            Task task = new Task().setCurrentPartNum(0).setStatus(Status.PAUSED).setName(name)
                    .setHash(hash).setSeeding(false).setTorrentPath(TORRENT_FILE_PATH + hash + ".torrent")
                    .setUploadType(uploadType)
                    .setSavePath(savePath) // savePath 需要去除末尾 /
                    .setUploadPath(uploadPath)
                    .setRatioLimit(ratioLimit)
                    .setSeedingTimeLimit(seedingTimeLimit)
                    .setMaxSize(maxSize * 1024 * 1024 * 1024); // 单位为 GB
                    
            try (FileInputStream fis = new FileInputStream(TORRENT_FILE_PATH + hash + ".torrent")) {
                Bencode bencode = new Bencode();
                byte[] data = fis.readAllBytes();
                Map<String, Object> torrentData = bencode.decode(data, com.dampcake.bencode.Type.DICTIONARY);
                Object infoObj = torrentData.get("info");
                @SuppressWarnings("unchecked")
                Map<String, Object> info = (infoObj instanceof Map) ? (Map<String, Object>) infoObj : new HashMap<>();
                String rootDir = (String) info.get("name");
                @SuppressWarnings("unchecked")
                List<LinkedHashMap<String, Object>> files = (ArrayList<LinkedHashMap<String, Object>>) info.get("files");
                int size = files.size();
                List<Long> fileLengths = files.stream().map(file -> {
                    return (Long) file.get("length");
                }).toList();
                List<List<Integer>> order = getTaskOrder(fileLengths, task.getMaxSize());
                task.setRootDir(rootDir).setFileNum(size).setTotalPartNum(order.size()).setTaskOrder(order);
            } catch (Exception e) {
                log.error("添加任务失败: {}", e.getMessage(), e);
                throw new RuntimeException(e);
            }

            Thread.sleep(1000);
            boolean setNotDownload = QbUtil.setNotDownload(task);
            Assert.isTrue(setNotDownload, "设置不下载失败");
            QbUtil.setPrio(hash, 1, task.getTaskOrder().get(0));
            QbUtil.setShareLimit(hash, ratioLimit, seedingTimeLimit);
            QbUtil.start(hash);
            task.setStatus(Status.DOWNLOADING);
            TASK_LIST.put(hash, task);
            log.info("添加任务成功: {}", task.getName());
            sync();
        } catch (Exception e) {
            log.error("添加任务失败: {}", e.getMessage(), e);
            throw new RuntimeException(e);
        }
    }

    /**
     * 从文件获取任务列表
     */
    @Synchronized("TASK_LIST")
    public static void load() {
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
    @Synchronized("TASK_LIST")
    public static void sync() {
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
    @Synchronized("TASK_LIST")
    public static void delete(String hash) {
        TASK_LIST.remove(hash);
        FileUtil.del(new File(TORRENT_FILE_PATH + hash + ".torrent"));
        QbUtil.delete(hash, true);
        sync();
        log.info("删除任务成功: {}", hash);
    }

    /**
     * 检测任务是否可在最大分片大小范围内完成
     * 
     * @param torrentContents
     * @param maxSize
     * @return 是否可以完成
     */
    public static boolean check(List<Long> torrentContents, long maxSize) {
        for (Long torrentContent : torrentContents) {
            if (torrentContent > maxSize) {
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
    public static List<List<Integer>> getTaskOrder(List<Long> torrentContentList, long maxSize)
            throws Exception {
        if (!check(torrentContentList, maxSize)) {
            throw new IllegalArgumentException("任务过大，无法分片");
        }
        List<List<Integer>> TaskOrder = new ArrayList<>();
        List<Integer> onePart = new ArrayList<>();
        long currentSize = 0;
        for (int i = 0, size = torrentContentList.size(); i < size; i++) {
            long torrentContent = torrentContentList.get(i);
            if (currentSize + torrentContent > maxSize) {
                TaskOrder.add(onePart);
                onePart = new ArrayList<>();
                currentSize = 0;
            }
            onePart.add(i);
            if (i == size - 1) {
                TaskOrder.add(onePart);
            }
            currentSize += torrentContent;
        }
        return TaskOrder;
    }
}