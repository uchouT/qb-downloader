package moe.uchout.qbdownloader.api;

import java.io.IOException;

import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;

@Auth
@Path("/test")
public class Test implements BaseAction {
    @Override
    public void doAction(HttpServerRequest arg0, HttpServerResponse arg1) throws IOException {
        resultSuccess();
    }
}
