package uchout.qbdownloader.util;

import uchout.qbdownloader.entity.Config;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public class ConfigUtil {
    public static final Config CONFIG = new Config();
    static {
        CONFIG.setQbHost("")
                .setQbUsername("")
                .setQbPassword("")
                .setAlistHost("")
                .setAlistToken("")
                .setCustomDownloadOrder(false)
                .setRcloneHost("http://localhost:5572")
                .setRclonePassword("secret")
                .setRcloneuserName("admin");
    }

    // FIXME: 后续设置无法更改
    /**
     * 加载配置文件
     */
    public static void load() {

    }
}