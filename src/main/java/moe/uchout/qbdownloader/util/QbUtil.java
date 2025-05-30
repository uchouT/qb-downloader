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
     * @return 是否登录成功
     */
    public static boolean login() {
        try {
            host = ConfigUtil.CONFIG.getQbHost();
            String username = ConfigUtil.CONFIG.getQbUsername();
            Assert.notBlank(username);
            String password = ConfigUtil.CONFIG.getQbPassword();
            Assert.notBlank(password);
            return HttpRequest.post(host + "/api/v2/auth/login")
                    .form("username", username)
                    .form("password", password)
                    .setFollowRedirects(true)
                    .thenFunction(res -> {
                        String body = res.body();
                        Assert.isTrue(res.isOk() && "Ok.".equals(body));
                        return true;
                    });
        } catch (Exception e) {
            log.warn("qbittorrent login failed.");
            return false;
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

    public static void setShareLimit(String hash, float ratioLimit, int seedingTimeLimit) throws QbException {
        try {
            HttpRequest.post(host + "/api/v2/torrents/setShareLimits")
                    .form("hashes", hash)
                    .form("ratioLimit", ratioLimit)
                    .form("seedingTimeLimit", seedingTimeLimit)
                    .form("inactiveSeedingTimeLimit", -2)
                    .then(res -> {
                        Assert.isTrue(res.isOk(), "set share limit failed, status code: {}", res.getStatus());
                        log.debug("set share limit for hash: {}, ratioLimit: {}, seedingTimeLimit: {}", hash,
                                ratioLimit,
                                seedingTimeLimit);
                    });
        } catch (Exception e) {
            log.error("set share limit failed: {}", e.getMessage(), e);
            throw new QbException(e.getMessage());
        }
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
                Thread.sleep(1000);
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
    public static boolean delete(String hash, boolean deleteFiles) {
        try {
            return HttpRequest.post(host + "/api/v2/torrents/delete")
                    .form("hashes", hash)
                    .form("deleteFiles", deleteFiles)
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
     * 根据链接添加种子，可以是磁力链接，也可以是本地文件路径链接
     * 
     * @param url      链接
     * @param savePath 保存路径
     * @param isFile   是否是本地文件路径
     * @return 是否添加成功
     */
    public static void add(String url, String savePath, boolean isFile) {
        HttpRequest req = HttpRequest.post(host + "/api/v2/torrents/add")
                // 所有通过 qb-downloader 添加的种子都属于这个分类
                .form("category", TorrentsInfo.category)
                .form("savepath", savePath);
        try {
            if (isFile) {
                req.form("torrents", new File(url))
                        .form("stopped", "true")
                        .then(res -> {
                            Assert.isTrue(res.isOk(), "add torrent failed, status code: {}", res.getStatus());
                        });
            } else {
                req.form("urls", url)
                        .form("tags", Tags.NEW)
                        .form("stopCondition", "MetadataReceived")
                        .then(res -> {
                            Assert.isTrue(res.isOk(), "add torrent failed, status code: {}", res.getStatus());
                        });
            }
        } catch (Exception e) {
            log.error(e.getMessage(), e);
        }
    }

    /**
     * 根据文件内容添加种子
     * 
     * @param torrentFile
     * @param fileName
     * @param savePath
     * @return
     */
    public static boolean add(byte[] torrentFile, String fileName, String savePath) {
        try {
            return HttpRequest.post(host + "/api/v2/torrents/add")
                    // 所有通过 qb-downloader 添加的种子都属于这个分类
                    .form("category", TorrentsInfo.category)
                    .form("savepath", savePath)
                    .form("torrents", torrentFile, fileName)
                    .form("stopped", "true")
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "add torrent failed, status code: {}", res.getStatus());
                        return true;
                    });

        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return false;
        }
    }
}
