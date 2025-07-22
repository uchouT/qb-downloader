<template>
    <div class="task-form">
        <el-form ref="formRef" :model="taskData" :rules="rules" label-position="right" label-width="140px">
            <el-form-item label="种子名称">
                <el-text>{{ taskData.torrentRes.torrentName }}</el-text>
            </el-form-item>
            <el-form-item label="上传路径">
                <el-input v-model="taskData.uploadPath" :placeholder="pathTips" />
            </el-form-item>
            <el-form-item label="文件大小限制" prop="maxSize">
                <el-input-number v-model="taskData.maxSize" precision="0" :min="1" :max="999">
                    <template #suffix>GB</template>
                </el-input-number>
            </el-form-item>
            <el-form-item label="上传工具" prop="uploadType">
                <el-radio-group v-model="taskData.uploadType">
                    <el-radio value="rclone">Rclone</el-radio>
                    <!-- <el-radio value="alist">Alist</el-radio> -->
                </el-radio-group>
            </el-form-item>
            <el-form-item label="做种时间限制">
                <el-input-number v-model="taskData.seedingTimeLimit" :placeholder="-2">
                    <template #suffix>分钟</template>
                </el-input-number>
            </el-form-item>
            <el-form-item label="分享率限制">
                <el-input-number v-model="taskData.ratioLimit" :precision="2" :placeholder="-2" :step="0.1" />
            </el-form-item>
        </el-form>
        <div class="warning-box flex-center">
            <p><strong>重要提醒：</strong>务必设置做种限制，否则分片任务将永远不会结束;<br>
                <small>-2 表示使用 qBittorrent 的全局设置</small>
            </p>
        </div>
        <div style="height: 20px;"></div>
        <div class="form-actions" style="text-align: right;">
            <el-button @click="emit('cancel')">取消</el-button>
            <el-button type="primary" :loading="loading" @click="handleSubmit">
                {{ loading ? '处理中...' : '完成' }}
            </el-button>
        </div>
    </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { ElMessage } from 'element-plus'

const taskData = defineModel('taskData')
const emit = defineEmits(['ok', 'cancel'])
const loading = ref(false)
const formRef = ref(null)
const pathTips = computed(() => {
    return taskData.uploadType === 'rclone' ? 'dest:/path/to/upload' : '/path/to/upload'
})
const rules = ref({
    uploadType: [{ required: true, message: '请选择上传工具', trigger: 'change' }],
    maxSize: [{ required: true, message: '请输入文件大小限制', trigger: 'blur' }]
})

const handleSubmit = async () => {
    if (!formRef.value) return

    try {
        await formRef.value.validate()
        loading.value = true
        emit('ok', () => {
            loading.value = false
        })
    } catch (error) {
        ElMessage.warning('请填写完整的信息')
    }
}

</script>

<style scoped>
.warning-box {
    background: rgba(255, 193, 7, 0.1);
    border: 1px solid rgba(255, 193, 7, 0.3);
    line-height: normal;
    border-radius: 6px;
    font-size: 16px;
}
</style>