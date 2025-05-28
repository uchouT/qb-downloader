package moe.uchout.qbdownloader.entity;

import lombok.Data;
import lombok.experimental.Accessors;

import java.io.Serializable;

/**
 * 设置项
 */
@Data
@Accessors(chain = true)
public class Config implements Serializable {
    /**
     * qbittorrent webUI 地址
     */
    private String qbHost;

    /**
     * qbittorrent 用户名
     */
    private String qbUsername;

    /**
     * qbittorrent 密码
     */
    private String qbPassword;

    /**
     * 所有任务消耗的总容量上限
     */
    private long totalSizeLimit;

    /**
     * Alist 地址
     */
    private String alistHost;

    /**
     * Alist Token
     */
    private String alistToken;

    /**
     * 是否自定义下载顺序 TODO
     */
    private boolean customDownloadOrder;

    /**
     * rclone rcd 地址
     */
    private String rcloneHost;

    /**
     * rclone rcd 用户名
     */
    private String rcloneuserName;

    /**
     * rclone rcd 密码
     */
    private String rclonePassword;

    /**
     * 仅允许内网
     */
    private boolean onlyInnerIP;

    /**
     * 是否校验 IP
     */
    private boolean verifyLoginIp;

    /**
     * 登录信息
     */
    private Login account;
}
