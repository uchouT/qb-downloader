package uchout.qbdownloader.util;

import com.google.gson.Gson;
import com.google.gson.JsonArray;

import cn.hutool.core.lang.Assert;
import cn.hutool.http.HttpRequest;
import lombok.extern.slf4j.Slf4j;

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
    public static synchronized Boolean login() {
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
     * 根据种子的 hash 获取种子信息
     * 
     * @param hash 种子信息 hash
     * @return 种子信息，以 JSON 格式返回
     */
    public static String getTorrentsInfo(String hash) {
        try {
            // HttpRequest.get(host + "/api/v2/torrents/info")
            //         .form("hashes", hash)
            //         .setFollowRedirects(true)
            //         .thenFunction(res -> {
            //             String body = res.body();
            //             JsonArray array = GsonStatic.gson.fromJson(body, JsonArray.class);
            //             return GsonStatic.fromJson(body, JsonArray.class);
            //         });
        } catch (Exception e) {
            log.error("qbittorrent get torrent info error: {}", e.getMessage());
            return "";
        }
    }
}
