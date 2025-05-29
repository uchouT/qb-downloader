package moe.uchout.qbdownloader.api.entity;

import java.io.Serializable;

import lombok.Data;
import lombok.experimental.Accessors;

@Data
@Accessors(chain = true)
public class TorrentRes implements Serializable {
    private String hash;
    private String savePath;
}
