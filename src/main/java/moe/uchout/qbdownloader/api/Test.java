package moe.uchout.qbdownloader.api;

import java.io.IOException;

import com.google.gson.JsonObject;

import cn.hutool.core.lang.Assert;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.util.GsonStatic;
import static moe.uchout.qbdownloader.util.uploader.Rclone.test;
import static moe.uchout.qbdownloader.util.QbUtil.login;
import static moe.uchout.qbdownloader.api.ConfigAction.rectifyHost;

@Auth
@Path("/test")
public class Test implements BaseAction {
    @Override
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
                    if (login(host, username, password)) {
                        resultSuccess();
                        return;
                    }
                    resultErrorMsg("qBittorrent 测试失败，请检查配置");
                    return;
                }
                if ("rclone".equals(type)) {
                    if (test(host, username, password)) {
                        resultSuccess();
                        return;
                    }
                    resultErrorMsg("Rclone 测试失败，请检查配置");
                    return;
                }
            } catch (Exception e) {
                resultErrorMsg("参数不完整");
                return;
            }
        }
        resultSuccess();
    }
}
