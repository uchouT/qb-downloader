package moe.uchout.qbdownloader.api.entity;

import java.io.Serializable;

import lombok.Data;
import lombok.experimental.Accessors;
import moe.uchout.qbdownloader.util.ConfigUtil;

@Data
@Accessors(chain = true)
public class TaskReq implements Serializable {
    private TorrentRes torrentRes;
    private String uploadType;
    private String uploadPath = ConfigUtil.CONFIG.getDefaultUploadPath();
    private long maxSize;
    private int seedingTimeLimit = ConfigUtil.CONFIG.getDefaultSeedingTimeLimit();
    private String ratioLimit = ConfigUtil.CONFIG.getDefaultRatioLimit();
}
