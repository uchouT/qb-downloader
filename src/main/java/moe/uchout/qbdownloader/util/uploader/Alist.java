package moe.uchout.qbdownloader.util.uploader;

import java.io.File;
import java.util.List;

import com.google.gson.JsonObject;

import cn.hutool.core.io.FileUtil;
import cn.hutool.core.lang.Assert;
import cn.hutool.core.util.URLUtil;
import cn.hutool.http.Header;
import cn.hutool.http.HttpConfig;
import cn.hutool.http.HttpRequest;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.util.ConfigUtil;
import moe.uchout.qbdownloader.util.GsonStatic;

/**
 * Alist 工具类
 */
@Slf4j
public class Alist implements Uploader {
    private Alist() {
    };

    private static final Alist INSTANCE = new Alist();

    public static Alist getInstance() {
        return INSTANCE;
    }

    /**
     * 使用 Alist 上传文件 TODO: 异常处理
     * TODO: Task 中保存文件路径
     * 
     * @param localPath  本地文件路径，需要绝对路径
     * @param remotePath 远程路径
     * @return 是否上传成功
     */
    @Override
    public boolean copy(String localPath, String remotePath) {
        try {
            String host = ConfigUtil.CONFIG.getAlistHost();
            String alistToken = ConfigUtil.CONFIG.getAlistToken();
            HttpConfig httpConfig = new HttpConfig()
                    .setBlockSize(1024 * 1024 * 50);
            if (FileUtil.isDirectory(localPath)) {
                List<File> files = FileUtil.loopFiles(localPath);
                for (File file : files) {
                    HttpRequest.put(host + "/api/fs/form")
                            .setConfig(httpConfig)
                            .timeout(1000 * 60 * 2)
                            .header(Header.AUTHORIZATION, alistToken)
                            .header("As-Task", "false")
                            .header(Header.CONTENT_LENGTH, String.valueOf(file.length()))
                            .header("File-Path", URLUtil.encode(remotePath + "/" + file.getPath()))
                            .form("file", file)
                            .then(res -> {
                                Assert.isTrue(res.isOk(), "上传失败 {} 状态码:{}", localPath, res.getStatus());
                                JsonObject jsonObject = GsonStatic.fromJson(res.body(), JsonObject.class);
                                int code = jsonObject.get("code").getAsInt();
                                log.debug(jsonObject.toString());
                                Assert.isTrue(code == 200, "上传失败 {} 状态码:{}", localPath, code);
                                log.info("Alist 上传文件成功: {} -> {}", file.getName(), remotePath + "/" + file.getPath());
                            });
                }
            } else {
                File file = new File(localPath);
                HttpRequest.put(host + "/api/fs/form")
                        .setConfig(httpConfig)
                        .timeout(1000 * 60 * 2)
                        .header(Header.AUTHORIZATION, alistToken)
                        .header("As-Task", "false")
                        .header(Header.CONTENT_LENGTH, String.valueOf(file.length()))
                        .header("File-Path", URLUtil.encode(remotePath + "/" + file.getName()))
                        .form("file", file)
                        .then(res -> {
                            Assert.isTrue(res.isOk(), "上传失败 {} 状态码:{}", localPath, res.getStatus());
                            JsonObject jsonObject = GsonStatic.fromJson(res.body(), JsonObject.class);
                            int code = jsonObject.get("code").getAsInt();
                            log.debug(jsonObject.toString());
                            Assert.isTrue(code == 200, "上传失败 {} 状态码:{}", localPath, code);
                            log.info("Alist 上传成功: {} -> {}", localPath, remotePath);
                        });
            }
            log.info("Alist 上传文件: {} -> {}", localPath, remotePath);
            return true;
        } catch (Exception e) {
            log.error("Alist 上传文件失败: {}", e.getMessage(), e);
            return false;
        }
    }

    /**
     * 检查 Alist 服务是否可用
     * 
     * @return 是否可用
     */
    @Override
    public boolean check() {
        try {
            // TODO
            return true;
        } catch (Exception e) {
            log.error("Alist 服务不可用", e);
            return false;
        }
    }
}
