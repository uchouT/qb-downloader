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

    private final static TaskThread taskThread = new TaskThread();

    public static void main(String[] args) {
        ARGS = List.of(ObjectUtil.defaultIfNull(args, new String[] {}));
        try {
            ConfigUtil.load();
            ServerUtil.start();
            taskThread.start();
            Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                Shutdown();
            }));
        } catch (Exception e) {
            log.error(e.getMessage());
            System.exit(1);
        }
    }

    public static synchronized void Shutdown() {
        log.info("Shutdown hook triggered, stopping server...");
        try {
            ConfigUtil.sync();
            log.info("Configuration synced.");
            taskThread.stopTask();
            log.info("Task thread stopped.");
            ServerUtil.stop();
            log.info("Server stopped.");
            log.info("Application shutdown completed successfully.");
        } catch (Exception e) {
            log.error("Error during shutdown: " + e.getMessage(), e);
        } finally {
            System.exit(0);
        }
    }
}

// TODO 日志记录需要整理，异常处理完善
// 所有路径末尾都没有 /