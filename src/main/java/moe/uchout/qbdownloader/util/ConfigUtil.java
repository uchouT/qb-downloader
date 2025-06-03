package moe.uchout.qbdownloader.util;

import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.entity.Config;
import moe.uchout.qbdownloader.entity.Login;

import java.io.FileWriter;

import cn.hutool.core.bean.BeanUtil;
import cn.hutool.core.bean.copier.CopyOptions;

import java.io.FileReader;
import java.io.File;
import org.yaml.snakeyaml.DumperOptions;
import org.yaml.snakeyaml.Yaml;
import org.yaml.snakeyaml.introspector.BeanAccess;
import org.yaml.snakeyaml.representer.Representer;

// TODO: 主程序密码需要加密
@Slf4j
public class ConfigUtil {
    public static final String CONFIG_DIR = System.getenv().getOrDefault("CONFIG", "config");
    public static final Config CONFIG = new Config();

    static {
        initConfigDirectories();
        String password = Md5Util.digestHex("adminadmin");
        CONFIG.setQbHost("http://localhost:8080")
                .setQbUsername("admin")
                .setQbPassword("adminadmin")
                .setAlistHost("http://localhost:5244")
                .setAlistToken("")
                .setRcloneHost("http://localhost:5572")
                .setRclonePassword("secret")
                .setRcloneUserName("admin")
                // .setTotalSizeLimit(0L)
                .setOnlyInnerIP(false)
                .setVerifyLoginIp(false)
                .setDefaultSavePath("")
                .setAccount(new Login().setUsername("admin").setPassword(password))
                .setDefaultRatioLimit("-2")
                .setDefaultSeedingTimeLimit(-2)
                .setDefaultUploadPath("");
    }

    /**
     * 初始化配置目录结构
     */
    private static void initConfigDirectories() {
        try {
            // 创建主配置目录
            File configDir = new File(CONFIG_DIR);
            if (!configDir.exists()) {
                if (configDir.mkdirs()) {
                    log.info("配置目录已创建: {}", configDir.getAbsolutePath());
                } else {
                    log.error("无法创建配置目录: {}", configDir.getAbsolutePath());
                    throw new RuntimeException("配置目录创建失败");
                }
            }

            if (!configDir.canRead() || !configDir.canWrite()) {
                log.error("配置目录权限不足: {}", configDir.getAbsolutePath());
                throw new RuntimeException("配置目录权限不足");
            }

            // 创建torrents子目录
            File torrentsDir = new File(configDir, "torrents");
            if (!torrentsDir.exists()) {
                if (torrentsDir.mkdirs()) {
                    log.info("Torrents目录已创建: {}", torrentsDir.getAbsolutePath());
                } else {
                    log.error("无法创建torrents目录: {}", torrentsDir.getAbsolutePath());
                    throw new RuntimeException("Torrents目录创建失败");
                }
            }

            log.debug("配置目录初始化完成: {}", configDir.getAbsolutePath());

        } catch (SecurityException e) {
            log.error("配置目录初始化失败，权限不足: {}", e.getMessage());
            throw new RuntimeException("配置目录初始化失败", e);
        } catch (Exception e) {
            log.error("配置目录初始化失败: {}", e.getMessage());
            throw new RuntimeException("配置目录初始化失败", e);
        }
    }

    /**
     * 加载配置文件
     */
    public static synchronized void load() {
        File file = new File(CONFIG_DIR, "config.yaml");
        if (!file.exists()) {
            log.debug("配置文件不存在, 使用默认配置");
            return;
        }
        try (FileReader reader = new FileReader(file)) {
            TaskUtil.load();
            DumperOptions options = new DumperOptions();
            options.setDefaultFlowStyle(DumperOptions.FlowStyle.BLOCK);
            options.setPrettyFlow(true);
            Representer representer = new Representer(options);
            representer.getPropertyUtils().setSkipMissingProperties(true);
            Yaml yaml = new Yaml(representer, options);
            yaml.setBeanAccess(BeanAccess.FIELD);
            Config loaded = yaml.loadAs(reader, Config.class);
            if (loaded != null) {
                BeanUtil.copyProperties(loaded, CONFIG, CopyOptions
                        .create()
                        .setIgnoreNullValue(true));
                log.info("加载配置文件成功");
            } else {
            }
        } catch (Exception e) {
            log.error("加载配置文件失败: {}", e.getMessage());
        }
    }

    /**
     * 保存配置文件
     */
    public static synchronized void sync() {
        try (FileWriter writer = new FileWriter(new File(CONFIG_DIR, "config.yaml"))) {
            DumperOptions options = new DumperOptions();
            options.setDefaultFlowStyle(DumperOptions.FlowStyle.BLOCK);
            options.setPrettyFlow(true);
            Representer representer = new Representer(options);
            representer.getPropertyUtils().setSkipMissingProperties(true);
            Yaml yaml = new Yaml(representer, options);
            yaml.setBeanAccess(BeanAccess.FIELD);
            String yamlStr = yaml.dumpAsMap(CONFIG);
            writer.write(yamlStr);
        } catch (Exception e) {
            log.error("保存配置文件失败", e);
        }
    }
}