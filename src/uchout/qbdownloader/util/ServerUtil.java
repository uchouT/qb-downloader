package uchout.qbdownloader.util;

import lombok.extern.slf4j.Slf4j;
import uchout.qbdownloader.Main;
import java.util.Map;

import cn.hutool.core.util.StrUtil;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import cn.hutool.http.server.SimpleServer;

@Slf4j
public class ServerUtil {
    public static final ThreadLocal<HttpServerRequest> REQUEST = new ThreadLocal<>();
    public static final ThreadLocal<HttpServerResponse> RESPONSE = new ThreadLocal<>();
    public static String HOST = "";
    public static String PORT = "7845";
    public static SimpleServer server;

    /**
     * 启动服务器
     */
    public static void start() {
        setHost();
    }

    /**
     * 设置服务器启动参数
     */
    static void setHost() {
        Map<String, String> env = System.getenv();
        int i = Main.ARGS.indexOf("-port");
        if (i > -1) {
            PORT = Main.ARGS.get(i + 1);
        }
        i = Main.ARGS.indexOf("-host");
        if (i > -1) {
            HOST = Main.ARGS.get(i + 1);
        }
        PORT = env.getOrDefault("PORT", PORT);
        HOST = env.getOrDefault("HOST", HOST);

        if (StrUtil.isBlank(HOST)) {
            server = new SimpleServer(Integer.parseInt(PORT));
        } else {
            try {
                server = new SimpleServer(HOST, Integer.parseInt(PORT));
            } catch (Exception e) {
                log.error(e.getMessage(), e);
                server = new SimpleServer(Integer.parseInt(PORT));
            }
        }
    }

    /**
     * 注册 API
     */
    // TODO: 学习注册 API
    // static void registerApi() {
    //     server.addAction("/", new RootAction());
    // }
}
