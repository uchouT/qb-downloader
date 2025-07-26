<template>
    <el-card shadow="hover">
        <template #header>
            <div class="task-name">{{ props.taskItem.name }}</div>
        </template>
        <el-row class="task-status" justify="space-between">
            <el-col :span="20"
                style="display: flex; flex-direction: column; gap: 10px; justify-content: space-between;">
                <div class="task-progress">
                    分片: {{ props.taskItem.currentPartNum + 1 }}/{{ props.taskItem.totalPartNum }}
                </div>
                <el-space class="task-status-tag">
                    <el-tag :type="getStatusType(props.taskItem.status)" size="small" style="text-align: right;">
                        {{ getStatusText(props.taskItem.status) }}
                    </el-tag>
                    <el-tag v-if="props.taskItem.seeding" type="primary" size="small">
                        做种中
                    </el-tag>
                </el-space>
                <div><el-progress :text-inside="true" :status="getProcessStatus(props.taskItem.status)"
                        :percentage="(props.taskItem.progress * 100).toFixed(2)">
                    </el-progress></div>

            </el-col>

            <el-col :span="2" style="display: flex; flex-direction: column; gap: 10px; justify-content: space-between;">
                <el-button plain size="small" v-if="props.taskItem.status === 'PAUSED'"
                    @click="startTask(props.taskItem.hash)" type="success" :loading="actionLoading[props.taskItem.hash]"
                    icon="VideoPlay">
                </el-button>
                <el-button plain size="small" v-if="props.taskItem.status === 'DOWNLOADING'"
                    @click="stopTask(props.taskItem.hash)" type="warning" :loading="actionLoading[props.taskItem.hash]"
                    icon="VideoPause">
                </el-button>
                <el-button plain size="small" @click="deleteTask(props.taskItem.hash)" type="danger"
                    :loading="actionLoading[props.taskItem.hash]" icon="Delete">
                </el-button>
                <el-button plain size="small" @click="showDetail = !showDetail" icon="InfoFilled"
                    type="info"></el-button>
            </el-col>
        </el-row>



        <template #footer v-if="showDetail">
            <div class="task-details" style="font-size: 14px;">
                <el-row>
                    <span class="label">种子名称:</span>
                    <span> {{ props.taskItem.name }}</span>
                </el-row>
                <el-row>
                    <span class="label">保存路径:</span>
                    <span>{{ props.taskItem.savePath }}</span>
                </el-row>
                <el-row>
                    <span class="label">上传路径:</span>
                    <span>{{ props.taskItem.uploadPath }}</span>
                </el-row>
                <el-row>
                    <span class="label">上传方式:</span>
                    <span>{{ props.taskItem.uploadType }}</span>
                </el-row>
            </div>
        </template>
    </el-card>
</template>

<script setup>
import { ref, reactive } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '../api'
const actionLoading = reactive({})

const props = defineProps(['taskItem'])
const emit = defineEmits(['refresh'])
const showDetail = ref(false)
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

const processStatusMap = {
    'ON_TASK': 'warning',
    'ALL_FINISHED': 'success',
    'ERROR': 'exception',
    'PAUSED': 'warning',
    'DOWNLOADED': 'success'
}

const getProcessStatus = (status) => {
    return processStatusMap[status];
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
.el-button {
    margin: 0;
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

.task-progress {
    font-size: 12px;
    color: var(--el-text-color-secondary);
}

.label {
    min-width: 80px;
    color: var(--el-text-color-secondary);
    font-weight: 500;
}
</style>