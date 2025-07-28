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
import moe.uchout.qbdownloader.api.entity.TorrentReq;
import moe.uchout.qbdownloader.api.entity.TorrentRes;
import moe.uchout.qbdownloader.exception.SingleFileException;
import moe.uchout.qbdownloader.util.ConfigUtil;
import moe.uchout.qbdownloader.util.QbUtil;
import moe.uchout.qbdownloader.util.TaskUtil;
import static moe.uchout.qbdownloader.api.ConfigAction.rectifyHost;
import static moe.uchout.qbdownloader.api.ConfigAction.rectifyPathAndHost;

@Slf4j
@Auth
@Path("/torrent")
/**
 * POST 通过链接添加种子,
 * multipart 通过文件添加种子, application/json 通过链接添加种子
 * 返回结果是 TorrentRes 对象, 种子添加完成后处于暂停状态，等待 TaskAction 设置相关参数
 * 
 * @see TorrentRes
 */
public class TorrentActon implements BaseAction {
    @Override
    public void doAction(HttpServerRequest req, HttpServerResponse res) throws IOException {
        try {
            Assert.isTrue(QbUtil.getLogin(), "Qb not login");
            String method = req.getMethod();
            if ("POST".equalsIgnoreCase(method)) {
                post(req);
                return;
            }
            if ("DELETE".equalsIgnoreCase(method)) {
                delete(req);
                return;
            }
            if ("GET".equalsIgnoreCase(method)) {
                get(req);
                return;
            }
            resultErrorMsg("Unsupported method: " + method);
            return;

        } catch (MissingParamException e) {
            String msg = e.getMessage();
            log.error("Error adding task: " + msg);
            resultErrorMsg(msg);
        } catch (SingleFileException e) {
            String msg = "不支持单文件种子";
            log.warn(msg);
            resultErrorMsg(msg);
        } catch (Exception e) {
            log.error("Error processing request: {}", e.getMessage(), e);
            resultErrorMsg(e.getMessage());
        }
    }

    /**
     * 处理种子任务添加. multipart 处理 .torrent 文件添加，否则处理 url 添加
     * 
     * @param req
     * @throws IOException
     */
    private void post(HttpServerRequest req) throws IOException {

        String savePath;
        boolean isMultipart = req.isMultipart();
        byte[] fileContent = null;
        String url;
        String torrentName;
        if (isMultipart) {
            MultipartFormData formData = req.getMultipart();
            UploadFile file = formData.getFile("torrent");
            Assert.notNull(file, "文件上传失败");
            while (!file.isUploaded()) {
                ThreadUtil.sleep(500);
            }
            savePath = rectifyHost(
                    getOptionalParam(formData, "savePath", ConfigUtil.CONFIG.getDefaultSavePath()));
            url = file.getFileName();
            fileContent = file.getFileContent();
        } else {
            TorrentReq torrentReq = getBody(TorrentReq.class);
            rectifyPathAndHost(torrentReq);
            savePath = torrentReq.getSavePath();
            url = torrentReq.getUrl();
        }
        Assert.notBlank(savePath, "保存路径不能为空。");
        String hash = TaskUtil.addTorrent(isMultipart, fileContent, url, savePath);
        torrentName = QbUtil.getName(hash);
        resultSuccess(new TorrentRes().setHash(hash).setSavePath(savePath).setTorrentName(torrentName));
    }

    /**
     * 删除种子
     * 
     * @param req
     * @throws MissingParamException
     */
    private void delete(HttpServerRequest req) throws MissingParamException {
        String hash = getRequiredParam(req, "hash");
        TaskUtil.delete(hash, false);
    }

    /**
     * 获取 torrent 内容树
     * 
     * @param req
     */
    private void get(HttpServerRequest req) throws MissingParamException, SingleFileException {
        String hash = getRequiredParam(req, "hash");
        resultSuccess(TaskUtil.getTorrentTree(hash));
    }
}