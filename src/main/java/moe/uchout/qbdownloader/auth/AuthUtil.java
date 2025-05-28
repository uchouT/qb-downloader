package moe.uchout.qbdownloader.auth;

import java.util.concurrent.TimeUnit;
import moe.uchout.qbdownloader.util.Md5Util;
import moe.uchout.qbdownloader.entity.Login;
import com.sun.net.httpserver.HttpExchange;
import moe.uchout.qbdownloader.entity.Result;
import moe.uchout.qbdownloader.api.BaseAction;
import moe.uchout.qbdownloader.entity.Config;
import cn.hutool.core.exceptions.ExceptionUtil;
import cn.hutool.core.util.ObjectUtil;
import cn.hutool.core.util.ReflectUtil;
import cn.hutool.core.util.StrUtil;
import cn.hutool.core.util.RandomUtil;
import cn.hutool.http.server.HttpServerRequest;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.util.ConfigUtil;
import moe.uchout.qbdownloader.util.GsonStatic;
import moe.uchout.qbdownloader.util.ServerUtil;

// TODO: 理解学习
@Slf4j
public class AuthUtil {
    private static final int loginEffectiveHours = 24;
    private static final boolean multiLoginForbidden = false;
    static {
        resetKey();
    }

    /**
     * 刷新有效时间
     */
    private static void resetTime() {
        String key = MyCacheUtil.get("auth_key");
        if (StrUtil.isBlank(key)) {
            return;
        }
        MyCacheUtil.put("auth_key", key, TimeUnit.HOURS.toMillis(loginEffectiveHours));
    }

    /**
     * 刷新密钥
     */
    public static String resetKey() {
        String key = "123";
        if (multiLoginForbidden) {
            // 禁止多端登录
            key = RandomUtil.randomString(128);
        }
        MyCacheUtil.put("auth_key", key, TimeUnit.HOURS.toMillis(loginEffectiveHours));
        return key;
    }

    /**
     * 根据登录信息生成 md5，并加上 key
     * 
     * @param login
     * @return MD5:key
     */
    public static String getAuth(Login login) {
        String key = MyCacheUtil.get("auth_key");
        if (StrUtil.isBlank(key)) {
            key = resetKey();
        }
        login.setKey(key);
        return Md5Util.digestHex(GsonStatic.toJson(login)) + ":" + key;
    }

    /**
     * 设置登录信息的 IP
     * 
     * @param login
     */
    public static void setIP(Login login) {
        if (ConfigUtil.CONFIG.isVerifyLoginIp()) {
            login.setIp(getIp());
        } else {
            login.setIp("");
        }
    }

    /**
     * 获取保存的登录信息
     * 
     * @return
     */
    public static Login getLogin() {
        Config config = ConfigUtil.CONFIG;
        Login login = ObjectUtil.clone(config.getAccount());
        setIP(login);
        return login;
    }

    public static boolean authorize(HttpServerRequest req) {
        String s = req.getHeader("Authorization");
        if (StrUtil.isBlank(s)) {
            BaseAction.staticResult(new Result<>().setCode(403).setMessage("未登录"));
            return false;
        }
        Login login = getLogin();
        String auth = getAuth(login);
        if (StrUtil.equals(auth, s)) {
            // 刷新有效时间
            resetTime();
            return true;
        }
        BaseAction.staticResult(new Result<>().setCode(403).setMessage("登录失效"));
        return false;
    }

    /**
     * 获取ip地址
     *
     * @return
     */
    public static String getIp() {
        try {
            HttpServerRequest request = ServerUtil.REQUEST.get();
            HttpExchange httpExchange = (HttpExchange) ReflectUtil.getFieldValue(request, "httpExchange");
            return httpExchange.getRemoteAddress().getAddress().getHostAddress();
        } catch (Exception e) {
            String message = ExceptionUtil.getMessage(e);
            log.error(message, e);
        }
        return "unknown";
    }
}
