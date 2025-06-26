package moe.uchout.qbdownloader.util.uploader;

import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.util.ConfigUtil;
import moe.uchout.qbdownloader.util.GsonStatic;
import moe.uchout.qbdownloader.entity.Task;
import moe.uchout.qbdownloader.enums.Status;

import com.google.gson.JsonObject;

import cn.hutool.core.lang.Assert;
import cn.hutool.http.ContentType;
import cn.hutool.http.Header;
import cn.hutool.http.HttpRequest;

import java.io.File;
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
     * @param dst
     * @return 是否上传成功
     * @see <a href=
     *      "https://rclone.org/rc/#running-asynchronous-jobs-with-async-true">rclone
     *      async</a>
     */
    @Override
    public void copy(Task task) {
        String host = ConfigUtil.CONFIG.getRcloneHost();
        String username = ConfigUtil.CONFIG.getRcloneUserName();
        String password = ConfigUtil.CONFIG.getRclonePassword();
        String src = task.getSavePath() + File.separator + task.getRootDir();
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

    @Override
    /**
     * 检查 rclone 任务状态
     * 
     * @param task
     * @return 是否上传完成
     * @see <a href=
     *      "https://rclone.org/rc/#job-status">rclone job status</a>
     */
    public boolean check(Task task) {
        String host = ConfigUtil.CONFIG.getRcloneHost();
        String username = ConfigUtil.CONFIG.getRcloneUserName();
        String password = ConfigUtil.CONFIG.getRclonePassword();
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
                    if (finished && !success) {
                        String message = jsonObject.get("error").getAsString();
                        log.error(message);
                        task.setStatus(Status.ERROR);
                        // TODO
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
                        log.info("Rclone version: {}", version);
                        return true;
                    });
        } catch (Exception e) {
            log.warn("Rclone 测试失败: {}", e.getMessage());
            return false;
        }
    }
}
