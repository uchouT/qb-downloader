package moe.uchout.qbdownloader.util;

import com.google.gson.JsonArray;
import moe.uchout.qbdownloader.exception.*;
import cn.hutool.core.io.FileUtil;
import cn.hutool.core.lang.Assert;
import cn.hutool.http.HttpRequest;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.entity.TorrentsInfo;
import moe.uchout.qbdownloader.entity.Task;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import moe.uchout.qbdownloader.enums.Tags;
import moe.uchout.qbdownloader.entity.Config;
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

    private static String host;

    /**
     * 读取配置文件的登录信息
     * @return 是否登录成功
     */
    public static boolean login() {
        Config config = ConfigUtil.CONFIG;
        host = config.getQbHost();
        String username = config.getQbUsername();
        String password = config.getQbPassword();
        return login(host, username, password);

    }

    /** 
    * @param host qBittorrent host, 末尾没有 "/"
    * @param username
    * @param password
    *
    * @return 是否成功
    */
    public static boolean login(String host, String username, String password) {
        try {
            Assert.notBlank(host);
            Assert.notBlank(username);
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
     * 获取带有 QBD 分类的种子的实时信息
     * 
     * @return 种子信息列表，没有种子信息返回空列表
     */
    public static List<TorrentsInfo> getTorrentsInfo() {
        try {
            return HttpRequest.get(host + "/api/v2/torrents/info")
                    .form("category", TorrentsInfo.category)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}, msg: {}", res.getStatus(), res.body());
                        List<TorrentsInfo> torrentsInfosList = new ArrayList<>();
                        JsonArray jsonArray = GsonStatic.fromJson(res.body(), JsonArray.class);
                        for (JsonElement jsonElement : jsonArray) {
                            JsonObject jsonObject = jsonElement.getAsJsonObject();
                            String hash = jsonObject.get("hash").getAsString();
                            String state = jsonObject.get("state").getAsString();
                            Float progress = jsonObject.get("progress").getAsFloat();

                            TorrentsInfo torrentsInfo = new TorrentsInfo();
                            torrentsInfo.setHash(hash)
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
     * 最新添加的种子带有 Tags.NEW 标签，带有此标签的种子确保只同时存在一个
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
                        String hash = jsonObject.get("hash").getAsString();
                        QbUtil.removeTag(hash, Tags.NEW);
                        return hash;
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return null;
        }
    }

    /**
     * 获取种子的状态，用于 export() 时检测元数据是否下载完成
     *
     * @param hash 需要获取状态的种子 hash
     * @return State 状态
     */
    private static String getState(String hash) {
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

    /**
     * 获取种子名称，种子名称和种子根目录文件夹名称可能不同
     *
     * @param hash
     * @return 种子名称
     */
    public static String getName(String hash) {
        try {
            return HttpRequest.get(host + "/api/v2/torrents/info")
                    .form("hashes", hash)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "get Name failed, status code: {}", res.getStatus());
                        JsonArray jsonArray = GsonStatic.fromJson(res.body(), JsonArray.class);
                        if (jsonArray.size() == 0) {
                            return "";
                        }
                        JsonObject jsonObject = jsonArray.get(0).getAsJsonObject();
                        return jsonObject.get("name").getAsString();
                    });
        } catch (Exception e) {
            log.error(e.getMessage(), e);
            return "";
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

    public static void setShareLimit(String hash, String ratioLimit, int seedingTimeLimit) throws QbException {
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
    public static void add(String url, String savePath) {
        try {
            HttpRequest.post(host + "/api/v2/torrents/add")
                    // 所有通过 qb-downloader 添加的种子都属于这个分类
                    .form("category", TorrentsInfo.category)
                    .form("savepath", savePath)
                    .form("urls", url)
                    .form("tags", Tags.NEW)
                    .form("stopCondition", "MetadataReceived")
                    .then(res -> {
                        Assert.isTrue(res.isOk(), "add torrent failed, status code: {}", res.getStatus());
                    });

        } catch (Exception e) {
            log.error(e.getMessage(), e);
        }
    }

    /**
     * 用于从保存的种子文件中快速添加种子
     * 
     * @param filePath
     * @param savePath
     * @param seedingTimeLimit
     * @param ratioLimit
     */
    public static void add(String filePath, String savePath, int seedingTimeLimit, String ratioLimit) {
        try {
            HttpRequest.post(host + "/api/v2/torrents/add")
                    // 所有通过 qb-downloader 添加的种子都属于这个分类
                    .form("category", TorrentsInfo.category)
                    .form("savepath", savePath)
                    .form("seedingTimeLimit", seedingTimeLimit)
                    .form("ratioLimit", ratioLimit)
                    .form("torrents", new File(filePath))
                    .form("stopped", "true")
                    .then(res -> {
                        Assert.isTrue(res.isOk(), "add torrent failed, status code: {}", res.getStatus());
                    });

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
                    .form("tags", Tags.NEW)
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
