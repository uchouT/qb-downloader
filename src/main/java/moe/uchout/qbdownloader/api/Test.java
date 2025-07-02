package moe.uchout.qbdownloader.api;

import java.io.IOException;

import com.google.gson.JsonObject;

import cn.hutool.core.lang.Assert;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.api.entity.TestRes;
import moe.uchout.qbdownloader.entity.Config;
import moe.uchout.qbdownloader.util.ConfigUtil;
import moe.uchout.qbdownloader.util.GsonStatic;
import moe.uchout.qbdownloader.util.QbUtil;

import static moe.uchout.qbdownloader.util.uploader.Rclone.test;
import static moe.uchout.qbdownloader.api.ConfigAction.rectifyHost;

@Auth
@Path("/test")
@Slf4j
public class Test implements BaseAction {
    @Override
    /**
     * 请求为 post 时，测试 qb 或者 rclone 状态
     * 请求为 get 时，返回 qb 和 rclone 设置状态
     * 没有请求方法时，直接返回成功，用于检测 QBD 登录情况
     */
    public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
        if ("POST".equalsIgnoreCase(req.getMethod())) {
            try {
                JsonObject json = GsonStatic.fromJson(req.getBody(), JsonObject.class);
                String type = json.get("type").getAsString();
                Assert.notBlank(type, "type cannot be blank");
                String host = json.get("host").getAsString();
                Assert.notBlank(host, "host cannot be blank");
                String username = json.get("username").getAsString();
                Assert.notBlank(username, "username cannot be blank");
                String password = json.get("password").getAsString();
                Assert.notBlank(password, "password cannot be blank");

                host = rectifyHost(host);

                if ("qb".equals(type)) {
                    if (QbUtil.login(host, username, password)) {
                        log.info("qBittorrent test success");
                        resultSuccess();
                        return;
                    }
                    String errorMsg = "qBittorrent test failed";
                    log.warn(errorMsg);
                    resultErrorMsg(errorMsg);
                    return;
                }
                if ("rclone".equals(type)) {
                    if (test(host, username, password)) {
                        resultSuccess();
                        return;
                    }
                    resultErrorMsg("Rclone test failed");
                    return;
                }
            } catch (Exception e) {
                resultErrorMsg("参数不完整");
                return;
            }
        }

        if ("GET".equalsIgnoreCase(req.getMethod())) {
            TestRes taskRes = new TestRes();
            Config config = ConfigUtil.CONFIG;
            String rcloneHost = config.getRcloneHost();
            String rcloneUserName = config.getRcloneUserName();
            String rclonePass = config.getRclonePassword();
            boolean rcloneOk = test(rcloneHost, rcloneUserName, rclonePass);
            taskRes.setUploaderOk(rcloneOk).setQbOk(QbUtil.getLogin());
            resultSuccess(taskRes);
            return;
        }
        resultSuccess();
    }
}
