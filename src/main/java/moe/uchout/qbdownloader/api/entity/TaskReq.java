package moe.uchout.qbdownloader.api.entity;

import java.io.Serializable;

import lombok.Data;
import lombok.experimental.Accessors;

@Data
@Accessors(chain = true)
public class TaskReq implements Serializable {
    private String url;
    private String savePath;
    private String uploadType;
    private String uploadPath;
    private int maxSize;
    private int seedingTimeLimit = 1440;
    private float ratioLimit = 1.0f;
}
