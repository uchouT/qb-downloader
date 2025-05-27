package moe.uchout.qbdownloader.action;

import java.util.Objects;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import java.util.function.Consumer;
import cn.hutool.core.io.IoUtil;
import cn.hutool.core.text.StrFormatter;
import cn.hutool.http.server.HttpServerResponse;
import cn.hutool.http.server.action.Action;
import moe.uchout.qbdownloader.util.GsonStatic;
import moe.uchout.qbdownloader.util.ServerUtil;
import moe.uchout.qbdownloader.entity.Result;

public interface BaseAction extends Action {
    Logger logger = LoggerFactory.getLogger(BaseAction.class);

    static <T> void staticResult(Result<T> result) {
        HttpServerResponse httpServerResponse = ServerUtil.RESPONSE.get();
        if (Objects.isNull(httpServerResponse)) {
            logger.error("httpServerResponse is null");
            return;
        }
        httpServerResponse.setContentType("application/json; charset=utf-8");
        String json = GsonStatic.toJson(result);
        IoUtil.writeUtf8(httpServerResponse.getOut(), true, json);
    }

    default <T> T getBody(Class<T> tClass) {
        return GsonStatic.fromJson(ServerUtil.REQUEST.get().getBody(), tClass);
    }

    default <T> void resultSuccess() {
        result(Result.success());
    }

    default <T> void resultSuccess(Consumer<Result<Void>> consumer) {
        Result<Void> success = Result.success();
        consumer.accept(success);
        result(success);
    }

    default <T> void resultSuccess(T t) {
        result(Result.success(t));
    }

    default <T> void resultSuccessMsg(String t, Object... argArray) {
        result(Result.success().setMessage(StrFormatter.format(t, argArray)));
    }

    default <T> void resultError() {
        result(Result.error());
    }

    default void resultError(Consumer<Result<Void>> consumer) {
        Result<Void> error = Result.error();
        consumer.accept(error);
        result(error);
    }

    default <T> void resultError(T t) {
        result(Result.error(t));
    }

    default <T> void resultErrorMsg(String t, Object... argArray) {
        result(Result.error().setMessage(StrFormatter.format(t, argArray)));
    }

    default <T> void result(Result<T> result) {
        staticResult(result);
    }
}
