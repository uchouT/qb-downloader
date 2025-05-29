package moe.uchout.qbdownloader.api;

import static moe.uchout.qbdownloader.api.ConfigAction.rectifyHost;
import java.io.IOException;

import cn.hutool.core.lang.Assert;
import cn.hutool.http.server.HttpServerRequest;
import cn.hutool.http.server.HttpServerResponse;
import lombok.extern.slf4j.Slf4j;
import moe.uchout.qbdownloader.annotation.Auth;
import moe.uchout.qbdownloader.annotation.Path;
import moe.uchout.qbdownloader.api.entity.TaskReq;
import moe.uchout.qbdownloader.util.QbUtil;
import moe.uchout.qbdownloader.util.TaskUtil;

@Slf4j
@Auth(value = false) // TODO: 仅测试阶段解除验证
@Path("/task")
/**
 * GET 获取任务状态
 * POST 管理任务 - 暂停, 开始/继续
 * PUT 通过 文件添加任务
 * DELETE 删除任务
 */
public class TaskAction implements BaseAction {
	@Override
	public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
		try {
			String method = req.getMethod();
			switch (method.toUpperCase()) {
				case "GET":

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
			resultErrorMsg("missing parameter" + e.getMessage());
			return;
		} catch (TaskException e) {
			resultErrorMsg("Task add failed." + e.getMessage());
		}
	}

	/**
	 * 管理任务
	 * 
	 * @param req
	 */
	private void put(HttpServerRequest req) throws MissingParamException {
		String type = getRequiredParam(req, "type");
		String hash = getRequiredParam(req, "hash");
		switch (type) {
			case "start":
				TaskUtil.start(hash);
				break;

			case "stop":
				TaskUtil.stop(hash);
				break;
			default:
				throw new MissingParamException("Unsupported type: " + type);
		}
	}

	/**
	 * 添加任务
	 * 
	 * @param req
	 */
	private void post(HttpServerRequest req) throws IOException, MissingParamException, TaskException {
		try {
			Assert.isTrue(QbUtil.login());
		} catch (Exception e) {
			log.error("Qb not login");
			throw new TaskException("Qb not login");
		}
		TaskReq taskReq = getBody(TaskReq.class);
		try {
			Assert.notNull(taskReq.getUploadType());
			Assert.notNull(taskReq.getTorrentRes().getSavePath());
			Assert.notNull(taskReq.getUploadPath());
			Assert.isTrue(taskReq.getMaxSize() > 0);
		} catch (Exception e) {
			throw new MissingParamException();
		}
		taskReq.setUploadPath(rectifyHost(taskReq.getUploadPath()));
		TaskUtil.addTask(taskReq.getTorrentRes(), taskReq.getUploadType(), taskReq.getUploadPath(),
				taskReq.getMaxSize(), taskReq.getSeedingTimeLimit(), taskReq.getRatioLimit());
	}

	private void delete(HttpServerRequest req) throws MissingParamException {
		String hash = getRequiredParam(req, "hash");
		TaskUtil.delete(hash);
	}
}