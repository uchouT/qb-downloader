package moe.uchout.qbdownloader;

import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.util.ConfigUtil;
import moe.uchout.qbdownloader.util.ServerUtil;
import moe.uchout.qbdownloader.util.TaskThread;
import cn.hutool.core.util.ObjectUtil;
import java.util.ArrayList;
import java.util.List;

@Slf4j
public class Main {
    public static List<String> ARGS = new ArrayList<>();

    public static void main(String[] args) {
        ARGS = List.of(ObjectUtil.defaultIfNull(args, new String[] {}));
        try {
            ConfigUtil.load();
            ServerUtil.start();
            new TaskThread().start();
            ;
        } catch (Exception e) {
            log.error(e.getMessage());
            System.exit(1);
        }
    }
}

// TODO 日志记录需要整理
// 所有路径末尾都没有 /