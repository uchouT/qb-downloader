package uchout.qbdownloader.util;

import com.google.gson.JsonArray;
import uchout.qbdownloader.entity.TorrentsInfo;
import cn.hutool.core.lang.Assert;
import cn.hutool.http.HttpRequest;
import lombok.extern.slf4j.Slf4j;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import uchout.qbdownloader.entity.TorrentContent;
import cn.hutool.core.util.StrUtil;

/**
 * Qbittorrent 种子下载相关
 */
@Slf4j
public class QbUtil {

    static String host;

    /**
     * 获取 host
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
    public static Boolean login() {
        try {
            getHost();
            String username = ConfigUtil.CONFIG.getQbUsername();
            if (username == null || username.isEmpty()) {
                throw new Exception("qbittorrent username is null");

            }
            String password = ConfigUtil.CONFIG.getQbPassword();
            if (password == null || password.isEmpty()) {
                throw new Exception("qbittorrent password is null");

            }
            return HttpRequest.post(host + "/api/v2/auth/login")
                    .form("username", username)
                    .form("password", password)
                    .setFollowRedirects(true)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}", res.getStatus());
                        String body = res.body();
                        Assert.isTrue("Ok.".equals(body), "body: {}", body);
                        return true;
                    });
        } catch (Exception e) {
            log.error("qbittorrent login error: {}", e.getMessage());
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
                            String eta = jsonObject.get("eta").getAsString();

                            TorrentsInfo torrentsInfo = new TorrentsInfo();
                            torrentsInfo.setHash(hash)
                                    .setSize(size)
                                    .setEta(eta)
                                    .setName(name)
                                    .setState(state)
                                    .setProgress(progress);
                            torrentsInfosList.add(torrentsInfo);
                        }
                        return torrentsInfosList;
                    });
        } catch (Exception e) {
            log.error(e.getMessage());
            return new ArrayList<>();
        }
    }

    /**
     * 重新校验种子
     * 
     * @param hash 种子 hash
     */
    public static void reAnnounce(String hash) {
        try {
            HttpRequest.post(host + "/api/v2/torrents/reannounce")
                    .form("hashes", hash)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}", res.getStatus());
                        return true;
                    });
        } catch (Exception e) {
            log.error(e.getMessage());
        }
    }

    /**
     * 设置种子的下载优先级
     * 
     * @param hash      种子的 hash
     * @param priority  优先级，1 表示下载， 0 表示不下载
     * @param indexList 需要设置优先级的种子分片索引
     */
    public static void setPrio(String hash, String priority, List<Integer> indexList) {
        try {
            String id = StrUtil.join("|", indexList);
            HttpRequest.post(host + "/api/v2/torrents/filePrio")
                    .form("hash", hash)
                    .form("priority", priority)
                    .form("id", id)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}", res.getStatus());
                        return true;
                    });

        } catch (Exception e) {
            log.error(e.getMessage());
        }
    }

    /**
     * 获取种子的内容信息
     * 
     * @param hash
     * @return ContentList 种子内容列表
     */
    public static List<TorrentContent> getContent(String hash) {
        try {
            return HttpRequest.get(host + "/api/v2/torrents/files")
                    .form("hash", hash)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), "status code: {}", res.getStatus());
                        List<TorrentContent> torrentContentList = new ArrayList<>();
                        JsonArray jsonArray = GsonStatic.fromJson(res.body(), JsonArray.class);
                        for (JsonElement jsonElement : jsonArray) {
                            JsonObject jsonObject = jsonElement.getAsJsonObject();
                            int index = jsonObject.get("index").getAsInt();
                            int size = jsonObject.get("size").getAsInt();
                            TorrentContent torrentContent = new TorrentContent();
                            torrentContent.setIndex(index).setSize(size);
                            torrentContentList.add(torrentContent);
                        }
                        return torrentContentList;
                    });
        } catch (Exception e) {
            log.error(e.getMessage());
            return new ArrayList<>();
        }
    }

    /**
     * 开始种子
     * 
     * @param hash 种子 hash
     */
    public static void start(String hash) {
    }

    /**
     * 暂停种子
     * 
     * @param hash 种子 hash
     */
    public static void pause(String hash) {
    }

    /**
     * 删除种子
     * 
     * @param hash 种子 hash
     */
    public static void delete(String hash) {
    }

    /**
     * 根据磁力链接添加种子
     * 
     * @param url 磁力链接
     */
    public static void add(String url) {
    }

    /**
     * 根据种子文件添加种子
     * TODO: 学习相关知识
     */
}
