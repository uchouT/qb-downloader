package moe.uchout.qbdownloader.api;

import java.io.IOException;

import cn.hutool.core.lang.Assert;
import cn.hutool.core.net.multipart.MultipartFormData;
import cn.hutool.core.net.multipart.UploadFile;
import cn.hutool.core.thread.ThreadUtil;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.api.entity.TaskReq;
import moe.uchout.qbdownloader.util.TaskUtil;

@Slf4j
@Auth
@Path("/task")
/**
 * POST 通过 url 添加任务
 * PUT 通过 文件添加任务
 * DELETE 删除任务
 */
public class Task implements BaseAction {
	@Override
	public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
		try {
			String method = req.getMethod();
			switch (method.toUpperCase()) {
				case "GET":
					// TODO
					return;
				case "POST":
					post(req);
					break;
				case "PUT":
					put(req);
					break;
				case "DELETE":
					delete(req);
					break;
				default:
					resultErrorMsg("Unsupported method: " + method);
					return;
			}
			resultSuccess();
		} catch (MissingParamException e) {
			resultErrorMsg("missing parameter");
			return;
		}
	}

	/**
	 * 通过 URL 添加任务
	 * 
	 * @param req
	 */
	private void post(HttpServerRequest req) throws MissingParamException {
		TaskReq taskReq = getBody(TaskReq.class);
		try {
			Assert.notNull(taskReq.getUrl());
			Assert.notNull(taskReq.getUploadType());
			Assert.notNull(taskReq.getSavePath());
			Assert.notNull(taskReq.getUploadPath());
			Assert.isTrue(taskReq.getMaxSize() > 0);
		} catch (Exception e) {
			throw new MissingParamException();
		}
		TaskUtil.addTask(taskReq.getUrl(), taskReq.getUploadType(), taskReq.getSavePath(), taskReq.getUploadPath(),
				taskReq.getMaxSize(), taskReq.getSeedingTimeLimit(), taskReq.getRatioLimit());
	}

	/**
	 * 通过文件添加任务
	 * 
	 * @param req
	 */
	private void put(HttpServerRequest req) throws IOException, MissingParamException {
		try {
			Assert.isTrue(req.isMultipart());
		} catch (Exception e) {
			throw new MissingParamException();
		}
		MultipartFormData formData = req.getMultipart();
		UploadFile file = formData.getFile("file");
		while (!file.isUploaded()) {
			ThreadUtil.sleep(500);
		}
		try {
			Assert.notNull(file);
		} catch (Exception e) {
			throw new MissingParamException();
		}
		String uploadType = getRequiredParam(req, "uploadType");
		String savePath = getRequiredParam(req, "savePath");
		String uploadPath = getRequiredParam(req, "uploadPath");
		int maxSize = Integer.parseInt(getRequiredParam(req, "maxSize"));
		int seedingTimeLimit = Integer.parseInt(getOptionalParam(req, "seedingTimeLimit", Default.seedingTimeLimit));
		float ratioLimit = Float.parseFloat(getOptionalParam(req, "ratioLimit", Default.ratioLimit));
		TaskUtil.addTask(file.getFileContent(), file.getFileName(), uploadType, savePath, uploadPath, maxSize,
				seedingTimeLimit, ratioLimit);
	}

	private void delete(HttpServerRequest req) throws MissingParamException {
		String hash = getRequiredParam(req, "hash");
		TaskUtil.delete(hash);
	}
}