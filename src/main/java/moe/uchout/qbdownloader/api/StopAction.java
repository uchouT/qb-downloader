package moe.uchout.qbdownloader.api;

import java.io.IOException;

import cn.hutool.core.thread.ThreadUtil;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import moe.uchout.qbdownloader.Main;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;

@Auth
@Path("/stop")
public class StopAction implements BaseAction {
    @Override
    public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
        try {
            resultSuccess("Shutdown hook triggered, stopping server...");
            new Thread(() -> {
                ThreadUtil.sleep(1000);
                Main.Shutdown();
            }, "shutdown-thread").start();

        } catch (Exception e) {
            resultErrorMsg("Error during shutdown: " + e.getMessage(), e);
        }
    }
}
