package moe.uchout.qbdownloader.api.entity;

import java.io.Serializable;

import lombok.Data;
import lombok.experimental.Accessors;

@Data
@Accessors(chain = true)
public class TestRes implements Serializable {
    /**
     * qb 是否登录
     */
    private Boolean qbOk;

    /**
     * uploader 状态
     */
    private Boolean uploaderOk;
}
