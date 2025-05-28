package moe.uchout.qbdownloader.entity;

import java.io.Serializable;

import lombok.Data;
import lombok.experimental.Accessors;

/**
 * 登录
 */
@Data
@Accessors(chain = true)
public class Login implements Serializable {
    /**
     * 用户名
     */
    private String username;
    /**
     * 密码
     */
    private String password;
    /**
     * ip
     */
    private String ip;
    /**
     * key
     */
    private String key;
}
