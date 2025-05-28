package moe.uchout.qbdownloader.action;

import java.io.IOException;

import cn.hutool.core.net.multipart.MultipartFormData;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.util.TaskUtil;

@Slf4j
@Auth
@Path("/task/delete")
public class TaskDeleteAction implements BaseAction {
    @Override
    public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
        if (req.isPostMethod() || req.isMultipart()) {
            MultipartFormData formData = req.getMultipart();
            try {
                String hash = getRequiredParam(formData, "hash");
                TaskUtil.delete(hash);
            } catch (MissingParamException e) {
                resultErrorMsg("missing parameter");
                return;
            }
        } else {
            resultErrorMsg("Invalid request method or content type");
            return;
        }
    }
}
