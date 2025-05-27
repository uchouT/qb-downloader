package moe.uchout.qbdownloader.auth;

import cn.hutool.core.io.FileUtil;
import cn.hutool.core.util.StrUtil;
import cn.hutool.json.JSONObject;
import cn.hutool.json.JSONUtil;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;

/**
 * Token 文件存储工具（可选，用于持久化存储）
 */
@Slf4j
public class TokenFileStorage {

    private static final String TOKEN_FILE = "data/token.json";
    private static final DateTimeFormatter FORMATTER = DateTimeFormatter.ISO_LOCAL_DATE_TIME;

    /**
     * 保存 token 到文件
     */
    public static void saveToken(String token, LocalDateTime expireTime) {
        try {
            JSONObject tokenData = new JSONObject();
            tokenData.set("token", token);
            tokenData.set("expireTime", expireTime != null ? expireTime.format(FORMATTER) : null);
            tokenData.set("createTime", LocalDateTime.now().format(FORMATTER));

            // 确保目录存在
            FileUtil.mkParentDirs(TOKEN_FILE);

            // 写入文件
            FileUtil.writeUtf8String(JSONUtil.toJsonPrettyStr(tokenData), TOKEN_FILE);
            log.info("Token saved to file: {}", TOKEN_FILE);

        } catch (Exception e) {
            log.error("Failed to save token to file", e);
        }
    }

    /**
     * 从文件加载 token
     */
    public static TokenData loadToken() {
        try {
            File file = new File(TOKEN_FILE);
            if (!file.exists()) {
                return null;
            }

            String content = FileUtil.readUtf8String(file);
            if (StrUtil.isBlank(content)) {
                return null;
            }

            JSONObject tokenData = JSONUtil.parseObj(content);
            String token = tokenData.getStr("token");
            String expireTimeStr = tokenData.getStr("expireTime");

            if (StrUtil.isBlank(token)) {
                return null;
            }

            LocalDateTime expireTime = null;
            if (StrUtil.isNotBlank(expireTimeStr)) {
                expireTime = LocalDateTime.parse(expireTimeStr, FORMATTER);

                // 检查是否过期
                if (LocalDateTime.now().isAfter(expireTime)) {
                    log.info("Loaded token is expired, deleting file");
                    deleteTokenFile();
                    return null;
                }
            }

            log.info("Token loaded from file: {}", TOKEN_FILE);
            return new TokenData(token, expireTime);

        } catch (Exception e) {
            log.error("Failed to load token from file", e);
            return null;
        }
    }

    /**
     * 删除 token 文件
     */
    public static void deleteTokenFile() {
        try {
            File file = new File(TOKEN_FILE);
            if (file.exists()) {
                FileUtil.del(file);
                log.info("Token file deleted: {}", TOKEN_FILE);
            }
        } catch (Exception e) {
            log.error("Failed to delete token file", e);
        }
    }

    /**
     * Token 数据类
     */
    public static class TokenData {
        private final String token;
        private final LocalDateTime expireTime;

        public TokenData(String token, LocalDateTime expireTime) {
            this.token = token;
            this.expireTime = expireTime;
        }

        public String getToken() {
            return token;
        }

        public LocalDateTime getExpireTime() {
            return expireTime;
        }
    }
}
