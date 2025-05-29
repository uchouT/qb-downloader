package moe.uchout.qbdownloader.api.entity;

import java.io.Serializable;

import lombok.Data;
import lombok.experimental.Accessors;
import moe.uchout.qbdownloader.util.ConfigUtil;

@Data
@Accessors(chain = true)
public class TorrentReq implements Serializable {
    private String url;
    private String savePath = ConfigUtil.CONFIG.getDefaultSavePath();
}
