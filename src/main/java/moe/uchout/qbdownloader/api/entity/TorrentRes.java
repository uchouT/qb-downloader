package moe.uchout.qbdownloader.api.entity;

import java.io.Serializable;

import lombok.Data;
import lombok.experimental.Accessors;
import moe.uchout.qbdownloader.util.ConfigUtil;

@Data
@Accessors(chain = true)
public class TorrentRes implements Serializable {
    private String hash;
    private String savePath = ConfigUtil.CONFIG.getDefaultSavePath();
}
