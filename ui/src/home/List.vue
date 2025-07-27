<template>
    <div class="task-list" style="height: 100%; display: flex; flex-direction: column;">
        <div class="list-header">
            <h3>任务列表</h3>
        </div>

        <div v-if="tasks.length === 0" class="flex-center" style="height: 100%;">
            <el-empty description="当前任务为空" />
        </div>

        <div v-else class="tasks-container flex-center" style="flex-direction:column; gap: 16px;">
            <ListItem v-for="task in tasks" :taskItem="task" :key="task.hash" @refresh="refreshTasks"
                :style="{ width: listWidth }" />
        </div>
    </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed } from 'vue'
import api from '../api'
import ListItem from './ListItem.vue'
import { useWindowSize } from '@vueuse/core'

// 响应式数据
const { width } = useWindowSize();
const listWidth = computed(() => {
    if (width.value <= 768) {
        return '95%'
    } else if (width.value <= 1024) {
        return '70%'
    } else {
        return '50%'
    }
})
const tasks = ref([])
const loading = ref(false)
let pollingTimer = null



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
</style>