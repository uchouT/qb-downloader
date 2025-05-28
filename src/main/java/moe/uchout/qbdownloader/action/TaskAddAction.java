package moe.uchout.qbdownloader.action;

import java.io.IOException;

import cn.hutool.core.net.multipart.MultipartFormData;
import cn.hutool.core.net.multipart.UploadFile;
import cn.hutool.core.thread.ThreadUtil;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.util.TaskUtil;

@Slf4j
@Auth
@Path("/task/add")
public class TaskAddAction implements BaseAction {
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
				int seedingTimeLimit = Integer
						.parseInt(getOptionalParam(formData, "seedingTimeLimit", Default.seedingTimeLimit));
				float ratioLimit = Float.parseFloat(getOptionalParam(formData, "ratioLimit", Default.ratioLimit));
				if (isFile) {
					UploadFile file = formData.getFile("file");
					while (!file.isUploaded()) {
						ThreadUtil.sleep(500);
					}
					if (file == null || file.getFileContent() == null) {
						resultErrorMsg("file is required");
						return;
					}
					TaskUtil.addTask(file.getFileContent(), file.getFileName(), uploadType, savePath, uploadPath,
							maxSize, seedingTimeLimit,
							ratioLimit);
				} else {
					String url = getRequiredParam(formData, "url");
					TaskUtil.addTask(url, uploadType, savePath, uploadPath, maxSize, seedingTimeLimit, ratioLimit);
				}
				resultSuccessMsg("Task added successfully");
				return;
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