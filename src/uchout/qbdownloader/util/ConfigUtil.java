package uchout.qbdownloader.util;

import uchout.qbdownloader.entity.Config;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public class ConfigUtil {
    public static final Config CONFIG = new Config();
    static {
        CONFIG.setQbHost(null)
                .setAlistHost(null)
                .setAlistToken(null)
                .setCustomDownloadOrder(false);
    }

    /**
     * 加载配置文件
     */
    public static void load() {

    }
}