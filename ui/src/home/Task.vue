<template>
    <div class="task-form">
        <el-form ref="formRef" :model="props.taskData" :rules="rules" label-position="right" label-width="140px">
            <el-form-item label="上传路径">
                <el-input v-model="props.taskData.uploadPath" :placeholder="pathTips" />
            </el-form-item>
            <el-form-item label="文件大小限制" prop="maxSize">
                <el-input-number v-model="props.taskData.maxSize" precision="0" :min="1" :max="999">
                    <template #suffix>GB</template>
                </el-input-number>
            </el-form-item>
            <el-form-item label="上传工具" prop="uploadType">
                <el-radio-group v-model="props.taskData.uploadType">
                    <el-radio value="rclone">Rclone</el-radio>
                    <!-- <el-radio value="alist">Alist</el-radio> -->
                </el-radio-group>
            </el-form-item>
            
            <div class="warning-box">
                <p><strong>重要提醒：</strong>务必设置做种限制，否则分片任务将永远不会结束</p>
                <p><small>-2 表示使用 qBittorrent 的全局设置</small></p>
            </div>
            
            <el-form-item label="做种时间限制">
                <el-input-number v-model="props.taskData.seedingTimeLimit" :placeholder="-2">
                    <template #suffix>分钟</template>
                </el-input-number>
            </el-form-item>
            <el-form-item label="分享率限制">
                <el-input-number v-model="props.taskData.ratioLimit" :precision="2" :placeholder="-2" :step="0.1" />
            </el-form-item>
        </el-form>
        
        <div class="form-actions">
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

const props = defineProps(['taskData'])
const emit = defineEmits(['ok', 'cancel'])
const loading = ref(false)
const formRef = ref(null)
const pathTips = computed(()=> {
    return props.taskData.uploadType === 'rclone' ? 'dest:/path/to/upload': '/path/to/upload'
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
.task-form {
    padding: 1.5rem;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
}

.el-form-item {
    margin-bottom: 1.5rem;
}

.warning-box {
    margin: 1rem 0;
    padding: 1rem;
    background: rgba(255, 193, 7, 0.1);
    border: 1px solid rgba(255, 193, 7, 0.3);
    border-radius: 6px;
    font-size: 14px;
}

.warning-box p {
    margin: 0 0 0.5rem 0;
}

.warning-box p:last-child {
    margin-bottom: 0;
}

.warning-box small {
    opacity: 0.8;
}

.form-actions {
    margin-top: 2rem;
    text-align: right;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    padding-top: 1rem;
}

.form-actions .el-button {
    margin-left: 0.5rem;
}

/* 移动端适配 */
@media (max-width: 768px) {
    .task-form {
        padding: 1rem;
    }
    
    .el-form :deep(.el-form-item__label) {
        width: 120px !important;
        text-align: left;
        font-size: 14px;
    }
    
    .el-form-item {
        margin-bottom: 1rem;
        flex-direction: column;
        align-items: stretch;
    }
    
    .el-form :deep(.el-form-item__content) {
        margin-left: 0 !important;
    }
    
    .el-form :deep(.el-input),
    .el-form :deep(.el-input-number) {
        width: 100%;
    }
    
    .el-form :deep(.el-radio-group) {
        display: flex;
        gap: 1rem;
    }
    
    .warning-box {
        font-size: 13px;
        padding: 0.875rem;
    }
    
    .form-actions {
        text-align: center;
        display: flex;
        gap: 0.75rem;
        justify-content: center;
    }
    
    .form-actions .el-button {
        flex: 1;
        margin-left: 0;
    }
}

@media (max-width: 480px) {
    .task-form {
        padding: 0.875rem;
    }
    
    .el-form :deep(.el-form-item__label) {
        width: 100px !important;
        font-size: 13px;
    }
    
    .warning-box {
        font-size: 12px;
        padding: 0.75rem;
    }
}
</style>