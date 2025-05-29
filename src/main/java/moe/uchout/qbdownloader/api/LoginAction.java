package moe.uchout.qbdownloader.api;

import java.io.IOException;

import cn.hutool.core.lang.Assert;
import cn.hutool.core.thread.ThreadUtil;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.auth.AuthUtil;
import moe.uchout.qbdownloader.entity.Login;

@Slf4j
@Auth(value = false)
@Path("/login")
public class LoginAction implements BaseAction {
    @Override
    public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
        Login newLogin = getBody(Login.class);
        Assert.notBlank(newLogin.getUsername(), "用户名不能为空");
        Assert.notBlank(newLogin.getPassword(), "密码不能为空");
        Login account = AuthUtil.getLogin();
        AuthUtil.setIP(newLogin);
        String username = newLogin.getUsername();
        String password = newLogin.getPassword();
        String accountUsername = account.getUsername();
        String accountPassword = account.getPassword();

        if (username.equals(accountUsername) && password.equals(accountPassword)) {
            AuthUtil.resetKey();
            log.info("登录成功");
            String s = AuthUtil.getAuth(newLogin);
            resultSuccess(s);
            return;
        }
        log.warn("登陆失败 {}, ip: {}", accountUsername, AuthUtil.getIp());
        ThreadUtil.sleep(4000);
        resultErrorMsg("用户名或密码错误");
    }

}
