<template>
    <el-card shadow="hover">
        <template #header>
            <div class="task-header">
                <div class="task-info">
                    <h4 class="task-name">{{ props.taskItem.name }}</h4>
                    <div class="task-meta">
                        <el-tag :type="getStatusType(props.taskItem.status)" size="small">
                            {{ getStatusText(props.taskItem.status) }}
                        </el-tag>
                        <el-tag v-if="props.taskItem.seeding" type="info" size="small">
                            做种中
                        </el-tag>
                        <span class="task-progress">
                            分片: {{ props.taskItem.currentPartNum + 1 }}/{{ props.taskItem.totalPartNum }}
                        </span>
                    </div>
                </div>
                <div class="task-actions">
                    <el-button-group size="small" plain>
                        <el-button v-if="props.taskItem.status === 'PAUSED'" @click="startTask(props.taskItem.hash)"
                            type="success" :loading="actionLoading[props.taskItem.hash]">
                            <el-icon>
                                <VideoPlay />
                            </el-icon>
                            开始
                        </el-button>
                        <el-button v-if="props.taskItem.status === 'DOWNLOADING'" @click="stopTask(props.taskItem.hash)"
                            type="warning" :loading="actionLoading[props.taskItem.hash]">
                            <el-icon>
                                <VideoPause />
                            </el-icon>
                            暂停
                        </el-button>
                        <el-button @click="deleteTask(props.taskItem.hash)" type="danger"
                            :loading="actionLoading[props.taskItem.hash]">
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
                <span class="value">{{ props.taskItem.savePath }}</span>
            </div>
            <div class="detail-row">
                <span class="label">上传路径:</span>
                <span class="value">{{ props.taskItem.uploadPath }}</span>
            </div>
            <div class="detail-row">
                <span class="label">上传方式:</span>
                <span class="value">{{ props.taskItem.uploadType }}</span>
            </div>
            <div class="detail-row">
                <span class="label">文件数量:</span>
                <span class="value">{{ props.taskItem.fileNum }} 个文件</span>
            </div>
        </div>
    </el-card>
</template>

<script setup>
import { ref, reactive } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '../api'
import { VideoPlay, VideoPause, Delete } from '@element-plus/icons-vue'
const actionLoading = reactive({})

const props = defineProps(['taskItem'])
const emit = defineEmits(['refresh'])
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
            emit('refresh')
            ElMessage.success(`task ${type} success`)
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
            emit('refresh')
            ElMessage.success('Task delete success')
        })
        .finally(() => {
            actionLoading[hash] = false
        })

}
</script>

<style scoped>
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
</style>