package moe.uchout.qbdownloader.auth;

import cn.hutool.core.util.StrUtil;
import cn.hutool.crypto.SecureUtil;
import cn.hutool.http.server.HttpServerRequest;
import moe.uchout.qbdownloader.action.BaseAction;
import moe.uchout.qbdownloader.entity.Result;

import java.time.LocalDateTime;


// TODO: 理解学习
public class AuthUtil {
    private static String TOKEN;
    private static LocalDateTime TOKEN_EXPIRE_TIME;

    // Token 默认有效期（小时）
    private static final int DEFAULT_EXPIRE_HOURS = 24;

    // Bearer 前缀
    private static final String BEARER_PREFIX = "Bearer ";

    /**
     * 生成新的 token
     * @return 生成的 token
     */
    public static String generateToken() {
        String tokenData = "" + System.currentTimeMillis() + Math.random();
        TOKEN = SecureUtil.md5(tokenData);
        TOKEN_EXPIRE_TIME = LocalDateTime.now().plusHours(DEFAULT_EXPIRE_HOURS);
        return TOKEN;
    }

    /**
     * 验证请求中的 token
     * 
     * @param req HTTP 请求
     * @return 是否验证通过
     */
    public static boolean authorize(HttpServerRequest req) {
        String authHeader = req.getHeader("Authorization");

        // 检查 Authorization header 是否存在
        if (StrUtil.isBlank(authHeader)) {
            sendUnauthorizedResponse("Missing Authorization header");
            return false;
        }

        // 提取 token
        String token = extractToken(authHeader);
        if (StrUtil.isBlank(token)) {
            sendUnauthorizedResponse("Invalid Authorization header format");
            return false;
        }

        // 验证 token
        if (!isValidToken(token)) {
            sendUnauthorizedResponse("Invalid or expired token");
            return false;
        }

        return true;
    }

    /**
     * 从 Authorization header 中提取 token
     * 
     * @param authHeader Authorization header 值
     * @return 提取的 token
     */
    private static String extractToken(String authHeader) {
        if (authHeader.startsWith(BEARER_PREFIX)) {
            return authHeader.substring(BEARER_PREFIX.length()).trim();
        }
        // 兼容直接传 token 的情况
        return authHeader.trim();
    }

    /**
     * 验证 token 是否有效
     * 
     * @param token 要验证的 token
     * @return 是否有效
     */
    private static boolean isValidToken(String token) {
        // 检查 token 是否存在
        if (TOKEN == null || StrUtil.isBlank(TOKEN)) {
            return false;
        }

        // 检查 token 是否匹配
        if (!TOKEN.equals(token)) {
            return false;
        }

        // 检查是否过期
        if (TOKEN_EXPIRE_TIME != null && LocalDateTime.now().isAfter(TOKEN_EXPIRE_TIME)) {
            // token 已过期，清除
            TOKEN = null;
            TOKEN_EXPIRE_TIME = null;
            return false;
        }

        return true;
    }

    /**
     * 撤销当前 token
     */
    public static void revokeToken() {
        TOKEN = null;
        TOKEN_EXPIRE_TIME = null;
    }

    /**
     * 检查是否有有效的 token
     * 
     * @return 是否有有效 token
     */
    public static boolean hasValidToken() {
        return TOKEN != null && (TOKEN_EXPIRE_TIME == null || LocalDateTime.now().isBefore(TOKEN_EXPIRE_TIME));
    }

    /**
     * 获取当前 token（用于登录接口返回）
     * 
     * @return 当前 token
     */
    public static String getCurrentToken() {
        return TOKEN;
    }

    /**
     * 获取 token 过期时间
     * 
     * @return 过期时间
     */
    public static LocalDateTime getTokenExpireTime() {
        return TOKEN_EXPIRE_TIME;
    }

    /**
     * 发送未授权响应
     * 
     * @param message 错误消息
     */
    private static void sendUnauthorizedResponse(String message) {
        BaseAction.staticResult(new Result<>()
                .setCode(401)
                .setMessage(message != null ? message : "Unauthorized"));
    }
}
