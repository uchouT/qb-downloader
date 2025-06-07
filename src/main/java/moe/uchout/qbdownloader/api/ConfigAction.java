package moe.uchout.qbdownloader.api;

import java.io.IOException;
import cn.hutool.core.bean.BeanUtil;
import cn.hutool.core.bean.copier.CopyOptions;
import cn.hutool.core.util.ObjectUtil;
import cn.hutool.core.util.StrUtil;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.entity.Config;
import moe.uchout.qbdownloader.util.ConfigUtil;
import java.lang.reflect.Field;

@Auth
@Path("/config")
public class ConfigAction implements BaseAction {
    @Override
    public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
        String method = req.getMethod();
        if (method.toUpperCase().equals("POST")) {
            Config config = ConfigUtil.CONFIG;
            String username = config.getAccount().getUsername();
            String password = config.getAccount().getPassword();
            BeanUtil.copyProperties(getBody(Config.class), config, CopyOptions
                    .create()
                    .setIgnoreNullValue(true));
            rectifyPathAndHost(config);
            String newUsername = config.getAccount().getUsername();
            if (StrUtil.isBlank(newUsername)) {
                config.getAccount().setUsername(username);
            }
            String newPassword = config.getAccount().getPassword();
            if (StrUtil.isBlank(newPassword)) {
                config.getAccount().setPassword(password);
            }
            ConfigUtil.sync();
            resultSuccessMsg("Configuration updated successfully.");
        } else if (method.toUpperCase().equals("GET")) {
            Config config = ObjectUtil.clone(ConfigUtil.CONFIG);
            config.getAccount().setPassword("");
            resultSuccess(config);
        } else {
            resultErrorMsg("Unsupported method: " + method);
            return;
        }
    }

    /**
     * 返回修正后的主机名
     * @param host
     * @return
     */
    static String rectifyHost(String host) {
        if (StrUtil.isNotBlank(host) && host.endsWith("/")) {
            return host.substring(0, host.length() - 1);
        }
        return host;
    }

    /**
     * 将对象中的路径和主机名进行修正，修正字段以 Path 或 Host 结尾
     * 
     * @param <T>
     * @param obj
     */
    static <T> void rectifyPathAndHost(T obj) {
        Field[] fields = obj.getClass().getDeclaredFields();
        try {
            for (Field field : fields) {
                if ((field.getName().endsWith("Host") || field.getName().endsWith("Path"))
                        && field.getType() == String.class) {
                    field.setAccessible(true);
                    String hostValue = (String) field.get(obj);
                    field.set(obj, rectifyHost(hostValue));
                }
            }
        } catch (IllegalAccessException e) {
            throw new RuntimeException("Failed to access field: " + e.getMessage(), e);
        }
    }
}
