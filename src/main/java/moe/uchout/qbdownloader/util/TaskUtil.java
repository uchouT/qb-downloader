package moe.uchout.qbdownloader.util;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.LinkedBlockingQueue;

import cn.hutool.core.thread.ExecutorBuilder;
import moe.uchout.qbdownloader.entity.Task;

/**
 * 任务执行相关
 */
public class TaskUtil {
    private static final ExecutorService EXECUTOR = ExecutorBuilder.create()
            .setCorePoolSize(1)
            .setMaxPoolSize(1)
            .setWorkQueue(new LinkedBlockingQueue<>(256))
            .build();
    /**
     * 任务列表
     */
    private static final List<Task> TASK_LIST = new ArrayList<>();

    /** 添加任务 */
    public static void addTask(Task task) {
        TASK_LIST.add(task);
    }

    /**
     * 获取任务列表
     */
    public static synchronized void load() {

    }

    /**
     * 同步任务列表, 将任务保存到文件中
     */
    public static synchronized void sync() {

    }

    /**
     * 删除任务，根据 id
     * 
     * @param id 任务 id
     */
    public static void delete(int id) {

    }

    /**
     * 异步运行间隔任务，成功发起间隔任务后，标记任务为 on-task, 
     * 间隔运行任务发生错误时，标记为 on-error
     * 
     * @param id 任务 id
     */
    public static void runIntervalTask(int id) {

    }
}
