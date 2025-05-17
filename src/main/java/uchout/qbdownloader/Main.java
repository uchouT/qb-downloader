package uchout.qbdownloader;

import lombok.extern.slf4j.Slf4j;
import cn.hutool.core.util.ObjectUtil;
import java.util.ArrayList;
import java.util.List;
import uchout.qbdownloader.util.ConfigUtil;

@Slf4j
public class Main {
    public static List<String> ARGS = new ArrayList<>();

    public static void main(String[] args) {
        ARGS = List.of(ObjectUtil.defaultIfNull(args, new String[] {}));
        try {
            ConfigUtil.load();
        } catch (Exception e) {
            log.error(e.getMessage());
            System.exit(1);
        }
    }
}

/**
 * 编写思路：
 * 1. 加载配置文件
 * 2. 启动服务器，并监听操作
 * 3. 核心：
 * 1. qBittorrent 添加任务到列表，每个任务占用一个线程，并记录到磁盘
 * 2. 任务配置：配置下载顺序，配置分片任务（rclone 上传，Alist 上传）
 * 3. 每一片下载完成后，更新任务数据，再执行分片任务
 * 4. 任务完成后，清除线程，但是任务记录留存，（可选删除）
 */