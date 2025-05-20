package moe.uchout.qbdownloader.util;

import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.entity.Config;
import java.io.FileWriter;
import cn.hutool.core.bean.BeanUtil;
import cn.hutool.core.bean.copier.CopyOptions;

import org.yaml.snakeyaml.DumperOptions;
import org.yaml.snakeyaml.Yaml;
import org.yaml.snakeyaml.introspector.BeanAccess;
import org.yaml.snakeyaml.representer.Representer;

@Slf4j
public class ConfigUtil {
    public static final Config CONFIG = new Config();
    static {
        CONFIG.setQbHost("http://localhost:8080")
                .setQbUsername("admin")
                .setQbPassword("adminadmin")
                .setAlistHost("http://localhost:5244")
                .setAlistToken("")
                .setCustomDownloadOrder(false)
                .setRcloneHost("http://localhost:5572")
                .setRclonePassword("secret")
                .setRcloneuserName("admin")
                .setTotalSizeLimit(0L);
    }

    // TODO: filename 还待考虑
    /**
     * 加载配置文件
     */
    public static synchronized void load() {
        try (java.io.FileReader reader = new java.io.FileReader("./config.yaml")) {
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
        try (FileWriter writer = new FileWriter("./config.yaml")) {
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