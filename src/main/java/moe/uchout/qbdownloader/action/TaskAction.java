package moe.uchout.qbdownloader.action;

import java.io.IOException;

import com.google.gson.JsonObject;

import cn.hutool.core.net.multipart.MultipartFormData;
import cn.hutool.core.net.multipart.UploadFile;
import cn.hutool.core.util.StrUtil;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.util.GsonStatic;
import moe.uchout.qbdownloader.util.TaskUtil;
import java.util.Map;

@Slf4j
@Auth
@Path("/task/add")
public class TaskAction implements BaseAction {
	@Override
	public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
		if (req.isPostMethod() && req.isMultipart()) {
			MultipartFormData formData = req.getMultipart();
			try {
				boolean isFile = Boolean.parseBoolean(getRequiredParam(formData, "isFile"));
				String uploadType = getRequiredParam(formData, "uploadType");
				String savePath = getRequiredParam(formData, "savePath");
				String uploadPath = getRequiredParam(formData, "uploadPath");
				int maxSize = Integer.parseInt(getRequiredParam(formData, "maxSize"));
				int seedingTimeLimit = Integer.parseInt(getOptionalParam(formData, "seedingTimeLimit", "1440"));
				float ratioLimit = Float.parseFloat(getOptionalParam(formData, "ratioLimit", "1.0f"));
				if (isFile) {
					UploadFile file = formData.getFile("file");
					// TODO: 学习 UploadFile 的用法
					TaskUtil.addTask(file, uploadType, savePath, uploadPath, maxSize, seedingTimeLimit, ratioLimit);
				} else {
					String url = getRequiredParam(formData, "url");
					TaskUtil.addTask(url, uploadType, savePath, uploadPath, maxSize, seedingTimeLimit, ratioLimit);
				}
			} catch (MissingParamException e) {
				resultErrorMsg("missing parameter");
				return;
			}
		}
	}
}
