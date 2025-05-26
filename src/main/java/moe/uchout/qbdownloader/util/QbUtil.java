package moe.uchout.qbdownloader.util;

import com.google.gson.JsonArray;
import moe.uchout.qbdownloader.exception.*;
import cn.hutool.core.io.FileUtil;
import cn.hutool.core.lang.Assert;
import cn.hutool.http.HttpRequest;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.entity.TorrentContent;
import moe.uchout.qbdownloader.entity.TorrentsInfo;
import moe.uchout.qbdownloader.entity.Task;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import moe.uchout.qbdownloader.enums.Tags;
import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.IntStream;
import java.util.stream.Collectors;

import cn.hutool.core.util.StrUtil;

// TODO: 重新设计错误处理机制
/**
 * Qbittorrent 种子下载相关
 */
@Slf4j
public class QbUtil {
    private QbUtil() {
    }

    private static String host;

    /**
     * 获取 host
     * TODO: 在配置 config 时，就把 / 去掉
     */
    static void getHost() throws Exception {
        host = ConfigUtil.CONFIG.getQbHost();
        if (host == null || host.isEmpty()) {
            throw new Exception("qbittorrent host is null");
        }
        if (host.endsWith("/")) {
            host = host.substring(0, host.length() - 1);
        }
    }

    /**
     * @return 是否登录成功
     */
    public static void login() throws QbException {
        try {
            getHost();
        } catch (Exception e) {
            throw new QbException("qbittorrent host is null");
        }
        String username = ConfigUtil.CONFIG.getQbUsername();
        if (username == null || username.isEmpty()) {
            throw new QbException("qbittorrent username is null");

        }
        String password = ConfigUtil.CONFIG.getQbPassword();
        if (password == null || password.isEmpty()) {
            throw new QbException("qbittorrent password is null");

        }
        boolean logined = HttpRequest.post(host + "/api/v2/auth/login")
                .form("username", username)
                .form("password", password)
                .setFollowRedirects(true)
                .thenFunction(res -> {
                    String body = res.body();
                    if (res.isOk() && "Ok.".equals(body)) {
                        log.info("qbittorrent login success");
                        return true;
                    }
                    return false;
                });
        if (!logined) {
            throw new QbException("qbittorrent login failed");
        }
    }

    /**
     * 获取种子的信息，带有 QBD 分类
     * 
     * @return 种子信息列表，没有种子信息返回空列表
     */
    public static List<TorrentsInfo> getTorrentsInfo() {
        try {
            return HttpRequest.get(host + "/api/v2/torrents/info")
                    .form("category", TorrentsInfo.category)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}");
                        List<TorrentsInfo> torrentsInfosList = new ArrayList<>();
                        JsonArray jsonArray = GsonStatic.fromJson(res.body(), JsonArray.class);
                        for (JsonElement jsonElement : jsonArray) {
                            JsonObject jsonObject = jsonElement.getAsJsonObject();
                            String hash = jsonObject.get("hash").getAsString();
                            String name = jsonObject.get("name").getAsString();
                            String state = jsonObject.get("state").getAsString();
                            Float progress = jsonObject.get("progress").getAsFloat();
                            int size = jsonObject.get("size").getAsInt();
                            int downloaded = jsonObject.get("downloaded").getAsInt();
                            String eta = jsonObject.get("eta").getAsString();

                            TorrentsInfo torrentsInfo = new TorrentsInfo();
                            torrentsInfo.setHash(hash)
                                    .setSize(size)
                                    .setDownloaded(downloaded)
                                    .setEta(eta)
                                    .setName(name)
                                    .setState(state)
                                    .setProgress(progress);
                            torrentsInfosList.add(torrentsInfo);
                        }
                        return torrentsInfosList;
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return new ArrayList<>();
        }
    }

    /**
     * 获取最新添加的种子的 hash
     * 
     * @return
     */
    public static String getHash() {
        try {
            return HttpRequest.get(host + "/api/v2/torrents/info")
                    .form("category", TorrentsInfo.category)
                    .form("tag", Tags.NEW)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "get Hash failed, status code: {}", res.getStatus());
                        JsonArray jsonArray = GsonStatic.fromJson(res.body(), JsonArray.class);
                        if (jsonArray.size() == 0) {
                            return null;
                        }
                        JsonObject jsonObject = jsonArray.get(0).getAsJsonObject();
                        return jsonObject.get("hash").getAsString();
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return null;
        }
    }

    public static String getState(String hash) {
        try {
            return HttpRequest.get(host + "/api/v2/torrents/info")
                    .form("hashes", hash)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "get Hash failed, status code: {}", res.getStatus());
                        JsonArray jsonArray = GsonStatic.fromJson(res.body(), JsonArray.class);
                        if (jsonArray.size() == 0) {
                            return null;
                        }
                        JsonObject jsonObject = jsonArray.get(0).getAsJsonObject();
                        return jsonObject.get("state").getAsString();
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return null;
        }
    }

    public static String getName(String hash) {
        try {
            return HttpRequest.get(host + "/api/v2/torrents/info")
                    .form("hashes", hash)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "get Name failed, status code: {}", res.getStatus());
                        JsonArray jsonArray = GsonStatic.fromJson(res.body(), JsonArray.class);
                        if (jsonArray.size() == 0) {
                            return null;
                        }
                        JsonObject jsonObject = jsonArray.get(0).getAsJsonObject();
                        return jsonObject.get("name").getAsString();
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return null;
        }
    }

    /**
     * 设置种子的下载优先级
     * 
     * @param hash      种子的 hash
     * @param priority  优先级，1 表示下载， 0 表示不下载
     * @param indexList 需要设置优先级的种子分片索引
     */
    public static boolean setPrio(String hash, Integer priority, List<Integer> indexList) {
        try {
            String id = StrUtil.join("|", indexList);
            return HttpRequest.post(host + "/api/v2/torrents/filePrio")
                    .form("hash", hash)
                    .form("priority", priority.toString())
                    .form("id", id)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "setPrio failed, status code: {}", res.getStatus());
                        return true;
                    });

        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return false;
        }
    }

    /**
     * 将 Task 中的种子文件设置为不下载
     * 
     * @param task
     * @return
     */
    public static boolean setNotDownload(Task task) {
        return setPrio(task.getHash(), 0, IntStream.range(0, task.getFileNum())
                .boxed()
                .collect(Collectors.toList()));
    }

    /**
     * 获取种子的内容信息，将获取到的 rootDir, files, fileNum 应用到任务实体中，
     * precondition: 种子内容符合规范，只有一个根目录，里面包含所有文件
     * size 的单位是字节
     * 
     * @param hash
     * @param task 任务实体
     * @return ContentList 种子内容列表
     */
    public static List<TorrentContent> getTorrentContentList(String hash, Task task) {
        try {
            return HttpRequest.get(host + "/api/v2/torrents/files")
                    .form("hash", hash)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}", res.getStatus());
                        List<TorrentContent> torrentContentList = new ArrayList<>();
                        List<String> files = new ArrayList<>();
                        JsonArray jsonArray = GsonStatic.fromJson(res.body(), JsonArray.class);
                        int fileNum = 0;
                        String savePath = task.getSavePath();
                        for (JsonElement jsonElement : jsonArray) {
                            JsonObject jsonObject = jsonElement.getAsJsonObject();
                            int index = jsonObject.get("index").getAsInt();
                            int size = jsonObject.get("size").getAsInt();
                            String path = jsonObject.get("name").getAsString();
                            TorrentContent torrentContent = new TorrentContent();
                            torrentContent.setIndex(index).setSize(size);
                            torrentContentList.add(torrentContent);
                            files.add(savePath + "/" + path);
                            fileNum++;
                        }
                        String rootDir = jsonArray.get(0).getAsJsonObject().get("name").getAsString().split("/")[0];
                        task.setRootDir(rootDir).setFileNum(fileNum).setFiles(files);
                        return torrentContentList;
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return new ArrayList<>();
        }
    }

    /**
     * 适用于种子管理操作
     * 
     * @param hash
     * @param req
     */
    private static boolean manage(String hash, String req) {
        try {
            return HttpRequest.post(host + "/api/v2/torrents/" + req)
                    .form("hashes", hash)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}", res.getStatus());
                        return true;
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return false;
        }
    }

    /**
     * 开始 / 继续 下载种子
     * 
     * @param hash 种子 hash
     */
    public static boolean start(String hash) {
        return manage(hash, "start");
    }

    /**
     * 暂停种子
     * NOTE: 种子做种状态下，暂停种子会直接完成种子，可能触发删除操作
     * 
     * @param hash 种子 hash
     */
    public static boolean pause(String hash) {
        return manage(hash, "stop");
    }

    /**
     * TODO: 可能用不上了，需要删除
     * 重新校验种子，涉及到文件内容删改的都应该 recheck
     * 
     * @param hash 种子 hash
     */
    public static boolean recheck(String hash) {
        return manage(hash, "recheck");
    }

    /**
     * 
     * @param hash
     * @param tag
     * @param req
     * @return
     */
    private static boolean tag(String hash, Tags tag, String req) {
        try {
            return HttpRequest.post(host + "/api/v2/torrents/" + req)
                    .form("hashes", hash)
                    .form("tags", tag.getTag())
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}", res.getStatus());
                        return true;
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return false;
        }
    }

    /**
     * 删除标签
     * 
     * @param hash
     * @param tag
     * @return
     */
    public static boolean removeTag(String hash, Tags tag) {
        return tag(hash, tag, "removeTags");
    }

    /**
     * 添加标签
     * 
     * @param hash
     * @param tag
     */
    public static boolean addTag(String hash, Tags tag) {
        return tag(hash, tag, "addTags");
    }

    /**
     * 将种子导出 .torrent 文件存放到指定位置
     * 
     * @param hash
     * @param path
     */
    public static synchronized void export(String hash, String path) {
        try {
            String state = getState(hash);
            // 直到元数据下载完成后，才导出种子
            while ("metaDL".equals(state)) {
                state = getState(hash);
                Thread.sleep(5000);
            }
            HttpRequest.post(host + "/api/v2/torrents/export")
                    .form("hash", hash)
                    .then(res -> {
                        Assert.isTrue(res.isOk(), "export torrents failed, status code: {}", res.getStatus());
                        FileUtil.writeBytes(res.bodyBytes(), new File(path));
                        log.debug("export torrent file to {}", path);
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
        }
    }

    /**
     * 删除种子
     * 
     * @param hash        种子 hash
     * @param deleteFiles 是否删除种子文件
     */
    public static boolean delete(String hash, Boolean deleteFiles) {
        try {
            return HttpRequest.post(host + "/api/v2/torrents/delete")
                    .form("hashes", hash)
                    .form("deleteFiles", deleteFiles.toString())
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "delete torrent failed, status code: {}", res.getStatus());
                        return true;
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return false;
        }
    }

    /**
     * 根据磁力链接添加种子，获取到 metadata 后暂停;
     * 会打上 "new" 标签，在处理完成后需要删除该标签
     * NOTE:刚添加后，所有文件默认都下载
     * 
     * @param url 磁力链接
     * @return 是否添加成功
     */
    public static boolean add(String url, boolean isFile) {
        try {
            HttpRequest req = HttpRequest.post(host + "/api/v2/torrents/add")
                    // 所有通过 qb-downloader 添加的种子都属于这个分类
                    .form("category", TorrentsInfo.category);
            if (isFile) {
                req.form("torrents", new File(url))
                        .form("stopped", "true");
            } else {
                req.form("urls", url)
                        .form("tags", Tags.NEW)
                        .form("stopCondition", "MetadataReceived");
            }
            return req.thenFunction(res -> {
                Assert.isTrue(res.isOk(), "add torrent failed, status code: {}", res.getStatus());
                return true;
            });

        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return false;
        }
    }
}
