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
    private static final String CONFIG_PATH = "configs";
    public static final Config CONFIG = new Config();
    static {
        String password = Md5Util.digestHex("adminadmin");
        CONFIG.setQbHost("http://localhost:8080")
                .setQbUsername("admin")
                .setQbPassword("adminadmin")
                .setAlistHost("http://localhost:5244")
                .setAlistToken("")
                .setCustomDownloadOrder(false)
                .setRcloneHost("http://localhost:5572")
                .setRclonePassword("secret")
                .setRcloneuserName("admin")
                .setTotalSizeLimit(0L)
                .setOnlyInnerIP(false)
                .setVerifyLoginIp(false)
                .setDefaultSavePath("")
                .setAccount(new Login().setUsername("admin").setPassword(password));
    }

    // TODO: filename 还待考虑
    /**
     * 加载配置文件
     */
    public static synchronized void load() {
        File file = new File(CONFIG_PATH + "/config.yaml");
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
            // TODO: 增加合法检验
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
        try (FileWriter writer = new FileWriter(CONFIG_PATH + "/config.yaml")) {
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