<template>
    <div class="task-form">
        <el-form ref="formRef" :model="props.taskData" :rules="rules" label-position="right" label-width="140px"
            class="aligned-form">
            <el-form-item label="upload path" prop="uploadPath">
                <el-input v-model="props.taskData.uploadPath" placeholder="输入上传路径" />
            </el-form-item>
            <el-form-item label="max size" prop="maxSize">
                <el-input-number v-model="props.taskData.maxSize" precision="2">
                    <template #suffix>GB</template>
                </el-input-number>
            </el-form-item>
            <el-form-item label="uploader" prop="uploadType">
                <el-radio-group v-model="props.taskData.uploadType">
                    <el-radio value="rclone">Rclone</el-radio>
                    <el-radio value="alist">Alist</el-radio>
                </el-radio-group>
            </el-form-item>
            <div class="warning-text">
                <el-text>务必设置做种限制，否则分片任务将永远不会结束<br> -2 使用 qBittorrent 的全局设置</el-text>
            </div>
            <el-form-item label="seeding time limit">
                <el-input-number v-model="props.taskData.seedingTimeLimit" placehold="-2"> <template #suffix>
                        分钟</template>
                </el-input-number>
            </el-form-item>
            <el-form-item label="ratio limit">
                <el-input-number v-model="props.taskData.ratioLimit" precision=2 placehold="-2" />
            </el-form-item>
        </el-form>
        <div class="form-actions">
            <span>
                <el-button @click="async () => { emit('cancel') }">取消</el-button>
                <el-button type="primary" :loading="loading" @click="handleSubmit">完成</el-button>
            </span>
        </div>
    </div>
</template>

<script setup>
import { ref } from 'vue'
import { ElMessage } from 'element-plus'

const props = defineProps(['taskData'])
const emit = defineEmits(['ok', 'cancel'])
const loading = ref(false)
const formRef = ref(null)

const rules = ref({
    uploadType: [{ required: true, message: '请选择上传方式', trigger: 'change' }],
    uploadPath: [{ required: true, message: '请输入上传路径', trigger: 'blur' }],
    maxSize: [{ required: true, message: '请输入最大大小', trigger: 'blur' }]
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
    padding: 16px;
}

.aligned-form {
    max-width: 600px;
}

.aligned-form .el-form-item {
    margin-bottom: 24px;
}

.warning-text {
    margin: 16px 0;
    padding: 12px;
    background-color: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 4px;
    color: inherit;
    font-size: 14px;
    line-height: 1.5;
}

.form-actions {
    margin-top: 24px;
    text-align: right;
}

/* 移动端适配 */
@media (max-width: 768px) {
    .task-form {
        padding: 12px 8px;
    }
    
    .aligned-form {
        max-width: 100%;
    }
    
    .aligned-form :deep(.el-form-item__label) {
        width: 120px !important;
        text-align: left;
        font-size: 14px;
        margin-bottom: 0.5rem;
    }
    
    .aligned-form .el-form-item {
        margin-bottom: 1rem;
        flex-direction: column;
        align-items: stretch;
    }
    
    .aligned-form :deep(.el-form-item__content) {
        margin-left: 0 !important;
    }
    
    .aligned-form :deep(.el-input),
    .aligned-form :deep(.el-input-number) {
        width: 100%;
    }
    
    .aligned-form :deep(.el-radio-group) {
        width: 100%;
        display: flex;
        gap: 1rem;
    }
    
    .warning-text {
        margin: 12px 0;
        padding: 10px;
        font-size: 13px;
    }
    
    .form-actions {
        margin-top: 1rem;
        text-align: center;
    }
    
    .form-actions span {
        display: flex;
        gap: 0.75rem;
        justify-content: center;
    }
    
    .form-actions :deep(.el-button) {
        flex: 1;
        font-size: 14px;
        padding: 10px 16px;
    }
}

@media (max-width: 480px) {
    .task-form {
        padding: 10px 6px;
    }
    
    .aligned-form :deep(.el-form-item__label) {
        width: 100px !important;
        font-size: 13px;
    }
    
    .aligned-form .el-form-item {
        margin-bottom: 0.875rem;
    }
    
    .warning-text {
        padding: 8px;
        font-size: 12px;
        margin: 10px 0;
    }
    
    .form-actions :deep(.el-button) {
        font-size: 13px;
        padding: 9px 14px;
    }
}

/* 平板适配 */
@media (min-width: 769px) and (max-width: 1024px) {
    .aligned-form :deep(.el-form-item__label) {
        width: 160px !important;
    }
}
</style>