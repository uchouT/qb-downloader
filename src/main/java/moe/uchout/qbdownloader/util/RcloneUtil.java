package moe.uchout.qbdownloader.util;

import lombok.extern.slf4j.Slf4j;

import com.google.gson.JsonObject;

import cn.hutool.core.lang.Assert;
import cn.hutool.http.HttpRequest;

@Slf4j
public class RcloneUtil {

    /**
     * rclone 上传文件
     * 
     * @param src
     * @param dst
     * @return 是否上传成功
     */
    public static boolean copy(String src, String dst) {
        String host = ConfigUtil.CONFIG.getRcloneHost();
        String username = ConfigUtil.CONFIG.getRcloneuserName();
        String password = ConfigUtil.CONFIG.getRclonePassword();
        JsonObject obj = new JsonObject();
        obj.addProperty("srcFs", src);
        obj.addProperty("dstFs", dst);
        obj.addProperty("createEmptySrcDirs", true);
        try {
            return HttpRequest.post(host + "/sync/copy")
                    .basicAuth(username, password)
                    .header("Content-Type", "application/json")
                    .body(GsonStatic.toJson(obj))
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), res.body());
                        log.info("rclone copied src: {}, dst: {}", src, dst);
                        return true;
                    });
        } catch (Exception e) {
            log.error("rclone copy error: {}", e.getMessage());
            return false;
        }
    }
}
