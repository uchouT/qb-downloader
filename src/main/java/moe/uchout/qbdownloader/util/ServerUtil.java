package moe.uchout.qbdownloader.util;

import java.net.InetSocketAddress;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import moe.uchout.qbdownloader.entity.Config;
import cn.hutool.core.io.IoUtil;
import cn.hutool.core.lang.PatternPool;
import cn.hutool.core.net.Ipv4Util;
import cn.hutool.core.net.NetUtil;
import cn.hutool.core.util.ClassUtil;
import cn.hutool.core.util.ReflectUtil;
import cn.hutool.core.util.StrUtil;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import cn.hutool.http.server.SimpleServer;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.Main;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.api.BaseAction;
import moe.uchout.qbdownloader.auth.AuthUtil;
import moe.uchout.qbdownloader.entity.Result;
import moe.uchout.qbdownloader.api.RootAction;
import static moe.uchout.qbdownloader.auth.AuthUtil.getIp;

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

        server.addFilter((req, res, chain) -> {
            REQUEST.set(req);
            RESPONSE.set(res);
            Config config = ConfigUtil.CONFIG;
            Boolean onlyInnerIP = config.isOnlyInnerIP();
            try {
                String ip = getIp();

                // 仅允许内网ip访问
                if (onlyInnerIP) {
                    if (!PatternPool.IPV4.matcher(ip).matches()) {
                        res.send404("404 Not Found");
                        return;
                    }
                    if (!Ipv4Util.isInnerIP(ip)) {
                        res.send404("404 Not Found");
                        return;
                    }
                }
                chain.doFilter(req.getHttpExchange());
            } finally {
                REQUEST.remove();
                RESPONSE.remove();
            }
        });
        registerApi();
        server.getRawServer().start();
        InetSocketAddress address = server.getAddress();
        log.info("Http Server listen on [{}:{}]", address.getHostName(), address.getPort());
        for (String ip : NetUtil.localIpv4s()) {
            log.info("http://{}:{}", ip, address.getPort());
        }
    }

    /**
     * 设置服务器启动参数
     */
    private static void setHost() {
        Map<String, String> env = System.getenv();
        int i = Main.ARGS.indexOf("--port");
        if (i > -1) {
            PORT = Main.ARGS.get(i + 1);
        }
        i = Main.ARGS.indexOf("--host");
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
    private static void registerApi() {
        server.addAction("/", new RootAction());
        Set<Class<?>> classes = ClassUtil.scanPackageByAnnotation("moe.uchout.qbdownloader.api", Path.class);
        for (Class<?> clazz : classes) {
            Path path = clazz.getAnnotation(Path.class);
            if (path == null) {
                continue;
            }
            Object action = ReflectUtil.newInstanceIfPossible(clazz);
            String urlPath = "/api" + path.value();
            server.addAction(urlPath, new BaseAction() {
                @Override
                public void doAction(HttpServerRequest req, HttpServerResponse res) {
                    try {
                        Auth auth = clazz.getAnnotation(Auth.class);
                        if (auth != null && auth.value()) {
                            if (!AuthUtil.authorize(req)) {
                                return;
                            }
                        }
                        BaseAction baseAction = (BaseAction) action;
                        baseAction.doAction(req, res);
                    } catch (Exception e) {
                        String json = GsonStatic.toJson(Result.error().setMessage(e.getMessage()));
                        IoUtil.writeUtf8(res.getOut(), true, json);
                        if (!(e instanceof IllegalArgumentException)) {
                            log.error("{} {}", urlPath, e.getMessage());
                            log.error(e.getMessage(), e);
                        }
                    }
                };
            });
        }
    }

    /**
     * 停止服务器
     */
    public static void stop() {
        if (Objects.isNull(server)) {
            return;
        }
        try {
            server.getRawServer().stop(0);
        } catch (Exception e) {
            log.error(e.getMessage(), e);
        }
    }
}
