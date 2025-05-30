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
import moe.uchout.qbdownloader.util.QbUtil;
import moe.uchout.qbdownloader.util.TaskUtil;
import static moe.uchout.qbdownloader.api.ConfigAction.rectifyHost;
import static moe.uchout.qbdownloader.api.ConfigAction.rectifyPathAndHost;

@Slf4j
@Auth(value = false) // TODO: 仅测试阶段解除验证
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
        String method = req.getMethod();
        try {
            Assert.isTrue("POST".equalsIgnoreCase(method));
            Assert.isTrue(QbUtil.login(), "Qb not login");
            if (req.isMultipart()) {
                Assert.isTrue(req.isMultipart());
                MultipartFormData formData = req.getMultipart();
                UploadFile file = formData.getFile("torrent");
                Assert.notNull(file);
                while (!file.isUploaded()) {
                    ThreadUtil.sleep(500);
                }
                String savePath = rectifyHost(getRequiredParam(formData, "savePath"));
                String hash = TaskUtil.addTorrent(true, file.getFileContent(), file.getFileName(), savePath);
                resultSuccess(new TorrentRes().setHash(hash).setSavePath(savePath));
            } else {
                TorrentReq torrentReq = getBody(TorrentReq.class);
                rectifyPathAndHost(torrentReq);
                String hash = TaskUtil.addTorrent(false, null, torrentReq.getUrl(), torrentReq.getSavePath());
                resultSuccess(new TorrentRes().setHash(hash).setSavePath(torrentReq.getSavePath()));
            }
        } catch (Exception e) {
            log.error("Error processing request: {}", e.getMessage(), e);
            resultErrorMsg(e.getMessage());
        }
    }
}