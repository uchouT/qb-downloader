<template>
    <div class="task-list">
        <div class="list-header">
            <h3>任务列表</h3>
        </div>

        <div v-if="tasks.length === 0" class="empty-state">
            <el-empty description="当前任务为空" />
        </div>

        <div v-else class="tasks-container">
            <el-card v-for="task in tasks" :key="task.hash" class="task-card" shadow="hover">
                <template #header>
                    <div class="task-header">
                        <div class="task-info">
                            <h4 class="task-name">{{ task.name }}</h4>
                            <div class="task-meta">
                                <el-tag :type="getStatusType(task.status)" size="small">
                                    {{ getStatusText(task.status) }}
                                </el-tag>
                                <el-tag v-if="task.seeding" type="info" size="small">
                                    做种中
                                </el-tag>
                                <span class="task-progress">
                                    分片: {{ task.currentPartNum + 1 }}/{{ task.totalPartNum }}
                                </span>
                            </div>
                        </div>
                        <div class="task-actions">
                            <el-button-group size="small">
                                <el-button v-if="task.status === 'PAUSED'" @click="startTask(task.hash)" type="success"
                                    :loading="actionLoading[task.hash]">
                                    <el-icon>
                                        <VideoPlay />
                                    </el-icon>
                                    开始
                                </el-button>
                                <el-button v-if="task.status === 'DOWNLOADING'" @click="stopTask(task.hash)"
                                    type="warning" :loading="actionLoading[task.hash]">
                                    <el-icon>
                                        <VideoPause />
                                    </el-icon>
                                    暂停
                                </el-button>
                                <el-button @click="deleteTask(task.hash)" type="danger"
                                    :loading="actionLoading[task.hash]">
                                    <el-icon>
                                        <Delete />
                                    </el-icon>
                                    删除
                                </el-button>
                            </el-button-group>
                        </div>
                    </div>
                </template>

                <div class="task-details">
                    <div class="detail-row">
                        <span class="label">保存路径:</span>
                        <span class="value">{{ task.savePath }}</span>
                    </div>
                    <div class="detail-row">
                        <span class="label">上传路径:</span>
                        <span class="value">{{ task.uploadPath }}</span>
                    </div>
                    <div class="detail-row">
                        <span class="label">上传方式:</span>
                        <span class="value">{{ task.uploadType }}</span>
                    </div>
                    <div class="detail-row">
                        <span class="label">文件数量:</span>
                        <span class="value">{{ task.fileNum }} 个文件</span>
                    </div>
                </div>
            </el-card>
        </div>
    </div>
</template>

<script setup>
import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { VideoPlay, VideoPause, Delete } from '@element-plus/icons-vue'
import api from '../api'

// 响应式数据
const tasks = ref([])
const loading = ref(false)
const actionLoading = reactive({})
let pollingTimer = null

// 状态映射
const statusTextMap = {
    'ON_TASK': '上传中',
    'FINISHED': '上传完成',
    'DOWNLOADED': '下载完成',
    'DOWNLOADING': '下载中',
    'ALL_FINISHED': '全部完成',
    'ERROR': '错误',
    'PAUSED': '已暂停'
}

const statusTypeMap = {
    'ON_TASK': 'warning',
    'FINISHED': 'success',
    'DOWNLOADED': 'success',
    'DOWNLOADING': 'primary',
    'ALL_FINISHED': 'success',
    'ERROR': 'danger',
    'PAUSED': 'info'
}

// 方法
const getStatusText = (status) => {
    return statusTextMap[status] || status
}

const getStatusType = (status) => {
    return statusTypeMap[status] || 'info'
}

// 获取任务列表
const fetchTasks = async () => {
    loading.value = true
    api.get('/api/task')
        .then(res => {
            if (res.data) {
                tasks.value = Object.values(res.data)
            } else {
                tasks.value = []
            }
        })
        .finally(() => {
            loading.value = false
        })

}

// 刷新任务列表
const refreshTasks = () => {
    fetchTasks()
}

// 开始任务
const startTask = async (hash) => {
    await task_(hash, 'start')
}

// 停止任务
const stopTask = async (hash) => {
    await task_(hash, 'stop')
}

const task_ = async (hash, type) => {
    actionLoading[hash] = true
    await api.put(`/api/task?type=${type}&hash=${hash}`)
        .then(async () => {
            ElMessage.success(`task ${type} success`)
            await fetchTasks()
        })
        .finally(() => {
            actionLoading[hash] = false
        })
}

// 删除任务
const deleteTask = async (hash) => {

    await ElMessageBox.confirm(
        '确定要删除这个任务吗？此操作不可恢复。',
        '确认删除',
        {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning',
        }
    )

    actionLoading[hash] = true
    await api.del(`/api/task?hash=${hash}`)
        .then(async () => {
            ElMessage.success('Task delete success')
            await fetchTasks()
        })
        .finally(() => {
            actionLoading[hash] = false
        })

}

// 开始轮询
const startPolling = () => {
    // 立即获取一次数据
    fetchTasks()

    // 每5秒轮询一次
    pollingTimer = setInterval(() => {
        fetchTasks()
    }, 5000)
}

// 停止轮询
const stopPolling = () => {
    if (pollingTimer) {
        clearInterval(pollingTimer)
        pollingTimer = null
    }
}

// 生命周期
onMounted(() => {
    startPolling()
})

onUnmounted(() => {
    stopPolling()
})

// 暴露方法给父组件
defineExpose({
    refreshTasks,
    startPolling,
    stopPolling
})
</script>

<style scoped>
.task-list {
    height: 100%;
    display: flex;
    flex-direction: column;
}

.list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
    padding: 0 8px;
}

.list-header h3 {
    margin: 0;
    color: var(--el-text-color-primary);
}

.empty-state {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
}

.tasks-container {
    flex: 1;
    overflow-y: auto;
    padding-right: 8px;
}

.task-card {
    margin-bottom: 16px;
}

.task-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
}

.task-info {
    flex: 1;
    min-width: 0;
}

.task-name {
    margin: 0 0 8px 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--el-text-color-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.task-meta {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
}

.task-progress {
    font-size: 12px;
    color: var(--el-text-color-secondary);
}

.task-actions {
    margin-left: 16px;
    flex-shrink: 0;
}

.task-details {
    margin-top: 16px;
}

.detail-row {
    display: flex;
    margin-bottom: 8px;
    font-size: 14px;
}

.detail-row:last-child {
    margin-bottom: 0;
}

.label {
    min-width: 80px;
    color: var(--el-text-color-secondary);
    font-weight: 500;
}

.value {
    color: var(--el-text-color-primary);
    word-break: break-all;
}

/* 滚动条样式 */
.tasks-container::-webkit-scrollbar {
    width: 6px;
}

.tasks-container::-webkit-scrollbar-track {
    background: var(--el-fill-color-lighter);
    border-radius: 3px;
}

.tasks-container::-webkit-scrollbar-thumb {
    background: var(--el-border-color-darker);
    border-radius: 3px;
}

.tasks-container::-webkit-scrollbar-thumb:hover {
    background: var(--el-border-color-dark);
}

/* 移动端适配 */
@media (max-width: 768px) {
    .task-header {
        flex-direction: column;
        gap: 0.75rem;
    }

    .task-info {
        width: 100%;
    }

    .task-name {
        font-size: 14px;
        line-height: 1.4;
        white-space: normal;
        overflow: visible;
        text-overflow: unset;
        word-break: break-word;
    }

    .task-meta {
        gap: 8px;
        justify-content: flex-start;
    }

    .task-actions {
        margin-left: 0;
        width: 100%;
    }

    .task-actions .el-button-group {
        width: 100%;
        display: flex;
    }

    .task-actions .el-button {
        flex: 1;
        justify-content: center;
    }

    .detail-row {
        flex-direction: column;
        gap: 4px;
    }

    .label {
        min-width: auto;
        font-size: 12px;
        font-weight: 600;
    }

    .value {
        font-size: 13px;
        padding-left: 8px;
    }

    .tasks-container {
        padding-right: 4px;
    }

    .task-card {
        margin-bottom: 12px;
    }
}

/* 超小屏适配 */
@media (max-width: 480px) {
    .list-header {
        padding: 0 4px;
        margin-bottom: 12px;
    }

    .list-header h3 {
        font-size: 16px;
    }

    .task-name {
        font-size: 13px;
    }

    .task-actions .el-button {
        font-size: 12px;
        padding: 6px 8px;
    }

    .task-details {
        margin-top: 12px;
    }

    .detail-row {
        margin-bottom: 6px;
    }

    .label {
        font-size: 11px;
    }

    .value {
        font-size: 12px;
    }
}

/* 平板适配 */
@media (min-width: 769px) and (max-width: 1024px) {
    .task-header {
        gap: 1.25rem;
    }

    .task-name {
        font-size: 15px;
    }

    .task-meta {
        gap: 10px;
    }
}

/* 桌面端优化 */
@media (min-width: 1025px) {
    .task-card:hover {
        transform: translateY(-2px);
        transition: transform 0.2s ease;
    }

    .task-name {
        font-size: 17px;
    }

    .task-actions .el-button {
        transition: all 0.2s ease;
    }
}
</style>