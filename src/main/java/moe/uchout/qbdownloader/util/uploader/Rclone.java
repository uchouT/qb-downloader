package moe.uchout.qbdownloader.util.uploader;

import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.entity.Config;
import moe.uchout.qbdownloader.util.ConfigUtil;
import moe.uchout.qbdownloader.util.GsonStatic;
import moe.uchout.qbdownloader.entity.Task;
import moe.uchout.qbdownloader.enums.Status;

import com.google.gson.JsonObject;

import cn.hutool.core.lang.Assert;
import cn.hutool.http.ContentType;
import cn.hutool.http.Header;
import cn.hutool.http.HttpRequest;

import java.util.Map;

@Slf4j
public class Rclone implements Uploader {
    private Rclone() {
    };

    private static final Rclone INSTANCE = new Rclone();

    public static Rclone getInstance() {
        return INSTANCE;
    }

    /**
     * rclone 上传文件，假设所有文件都在同一个目录下，
     * 或者只有一个单文件
     * 
     * @param task
     * @see <a href=
     *      "https://rclone.org/rc/#running-asynchronous-jobs-with-async-true">rclone
     *      async</a>
     */
    @Override
    public void copy(Task task) {
        Config config = ConfigUtil.CONFIG;
        String host = config.getRcloneHost();
        String username = config.getRcloneUserName();
        String password = config.getRclonePassword();
        String src = task.getSavePath() + "/" + task.getRootDir();
        String dst = task.getUploadPath() + "/" + task.getRootDir();
        Map<String, Object> obj = Map.of(
                "srcFs", src,
                "dstFs", dst,
                "_async", true,
                "createEmptySrcDirs", true);
        try {
            HttpRequest.post(host + "/sync/copy")
                    .basicAuth(username, password)
                    .header(Header.CONTENT_TYPE, ContentType.JSON.toString())
                    .body(GsonStatic.toJson(obj))
                    .then(res -> {
                        Assert.isTrue(res.isOk(), res.body());
                        JsonObject jsonObject = GsonStatic.fromJson(res.body(), JsonObject.class);
                        int jobid = jsonObject.get("jobid").getAsInt();
                        task.setRcloneJobId(jobid);
                    });
        } catch (Exception e) {
            log.error("rclone copy error: {}", e.getMessage());
            task.setStatus(Status.ERROR);
        }
    }

    /**
     * @see <a href=
     *      "https://rclone.org/rc/#job-status">rclone job status</a>
     */
    @Override
    public boolean check(Task task) {
        Config config = ConfigUtil.CONFIG;
        String host = config.getRcloneHost();
        String username = config.getRcloneUserName();
        String password = config.getRclonePassword();
        int jobId = task.getRcloneJobId();

        Map<String, Object> obj = Map.of(
                "jobid", jobId);

        return HttpRequest.post(host + "/job/status")
                .basicAuth(username, password)
                .header(Header.CONTENT_TYPE, ContentType.JSON.toString())
                .body(GsonStatic.toJson(obj))
                .thenFunction(res -> {
                    Assert.isTrue(res.isOk(), res.body());
                    JsonObject jsonObject = GsonStatic.fromJson(res.body(), JsonObject.class);
                    boolean success = jsonObject.get("success").getAsBoolean();
                    boolean finished = jsonObject.get("finished").getAsBoolean();

                    // finished 和 success 值不相等时，代表上传出错
                    if (finished && !success) {
                        String message = jsonObject.get("error").getAsString();
                        log.error(message);
                        task.setStatus(Status.ERROR);
                        throw new RuntimeException("Rclone 任务失败: " + message);
                    }
                    return success;
                });
    }

    public static boolean test(String host, String username, String password) {
        try {
            return HttpRequest.post(host + "/core/version")
                    .basicAuth(username, password)
                    .thenFunction(res -> {
                        Assert.isTrue(res.isOk(), res.body());
                        JsonObject jsonObject = GsonStatic.fromJson(res.body(), JsonObject.class);
                        String version = jsonObject.get("version").getAsString();
                        log.info("Rclone test success, version: {}", version);
                        return true;
                    });
        } catch (Exception e) {
            log.warn("Rclone test failed: {}", e.getMessage());
            return false;
        }
    }
}
