<template>
    <el-dialog v-model="dialogVisible" title="设置" :width="dialogWidth" class="config-dialog">
        <div v-loading="loading">
            <el-tabs v-model="activeName" type="card" class="demo-tabs" @tab-click="handleClick">
                <el-tab-pane label="Basic" name="first">
                    <el-scrollbar>
                        <el-form label-position="right" label-width="140px">
                            <el-form-item label="QB Host">
                                <el-input v-model="config.qbHost" placeholder="http://localhost:8080" />
                            </el-form-item>
                            <el-form-item label="QB 用户名">
                                <el-input v-model="config.qbUsername" placeholder="admin" />
                            </el-form-item>
                            <el-form-item label="QB 密码">
                                <el-input v-model="config.qbPassword" type="password" placeholder="password" />
                            </el-form-item>
                            <el-form-item>
                                <el-button @click="test('qb')" bg text :loading="qbTestLoading" icon="Odometer">测试
                                </el-button>
                            </el-form-item>
                            <el-form-item label="仅限内网 IP">
                                <el-switch v-model="config.onlyInnerIP" />
                            </el-form-item>
                            <el-form-item label="IP 验证">
                                <el-switch v-model="config.verifyLoginIP" />
                            </el-form-item>
                            <el-form-item label="Username">
                                <el-input v-model="config.account.username" placeholder="admin" />
                            </el-form-item>
                            <el-form-item label="Password">
                                <el-input v-model="config.account.password" type="password" placeholder="password" />
                            </el-form-item>
                            <el-form-item label="默认分享率">
                                <el-input-number v-model="config.defaultRatioLimit" placeholder="-2" />
                            </el-form-item>
                            <el-form-item label="默认做种时间">
                                <el-input-number v-model="config.defaultSeedingTimeLimit" placeholder="-2">
                                    <template #suffix>分钟</template>
                                </el-input-number>
                            </el-form-item>
                        </el-form>
                    </el-scrollbar>
                </el-tab-pane>
                <el-scrollbar>
                    <el-tab-pane label="Uploader" name="second">
                        <el-form label-position="right" label-width="140px">
                            <el-form-item label="Rclone Host">
                                <el-input v-model="config.rcloneHost" placeholder="http://localhost:5572" />
                            </el-form-item>
                            <el-form-item label="Rclone Username">
                                <el-input v-model="config.rcloneUserName" placeholder="admin" />
                            </el-form-item>
                            <el-form-item label="Rclone Password">
                                <el-input v-model="config.rclonePassword" type="password" placeholder="secret" />
                            </el-form-item>
                            <el-form-item>
                                <el-button @click="test('rclone')" bg text :loading="rcloneTestLoading" icon="Odometer">测试
                                </el-button>
                            </el-form-item>
                            <el-form-item label="默认下载路径">
                                <el-input v-model="config.defaultSavePath" placeholder="/downloads" />
                            </el-form-item>
                            <el-form-item label="默认上传路径">
                                <el-input v-model="config.defaultUploadPath" placeholder="/uploads" />
                            </el-form-item>
                        </el-form>
                    </el-tab-pane>
                </el-scrollbar>
            </el-tabs>
        </div>
        <template #footer>
            <div class="dialog-footer">
                <el-button @click="dialogVisible = false">取消</el-button>
                <el-button type="primary" :loading="loading" @click="saveConfig">保存配置</el-button>
            </div>
        </template>
    </el-dialog>
</template>

<script setup>
import { ref, computed } from 'vue';
import { useWindowSize } from '@vueuse/core'
import api from '../api';
import CryptoJS from "crypto-js";
import { ElMessage } from 'element-plus';
const { width } = useWindowSize()

// 响应式对话框宽度
const dialogWidth = computed(() => {
    if (width.value <= 768) {
        return '95%'
    } else if (width.value <= 1024) {
        return '70%'
    } else {
        return '50%'
    }
})

const saveConfig = () => {
    loading.value = true
    let my_config = JSON.parse(JSON.stringify(config.value))
    if (my_config.account.password) {
        my_config.account.password = CryptoJS.MD5(my_config.account.password).toString();
    }
    api.post('api/config', my_config)
        .then(res => {
            ElMessage.success(res.message)
            emit('load')
            dialogVisible.value = false
        })
        .finally(() => {
            loading.value = false
        })
}
const qbTestLoading = ref(false)
const rcloneTestLoading = ref(false)
const dialogVisible = ref(false)
const activeName = ref('first')
const config = ref({
    qbHost: "",
    qbUsername: "",
    qbPassword: '',
    rcloneHost: '',
    rcloneUserName: '',
    rclonePassword: '',
    onlyInnerIP: false,
    verifyLoginIP: false,
    account: {
        username: '',
        password: ''
    },
    defaultSavePath: '',
    defaultUploadPath: '',
    defaultRatioLimit: -2,
    defaultSeedingTimeLimit: -2
})

const loading = ref(false)

const show = () => {
    dialogVisible.value = true
    loading.value = true
    api.get('api/config')
        .then(res => {
            config.value = res['data']
        })
        .finally(() => {
            loading.value = false
        })
}
const emit = defineEmits(['load'])
defineExpose({ show })

const test = (type) => {
    if (type === 'qb') {
        qbTestLoading.value = true
        api.post('api/test', {
            type: 'qb',
            host: config.value.qbHost,
            username: config.value.qbUsername,
            password: config.value.qbPassword
        }).then(res => {
            ElMessage.success(res.message)
        }).finally(() => {
            qbTestLoading.value = false
        })
        return
    }
    if (type === 'rclone') {
        rcloneTestLoading.value = true
        api.post('api/test', {
            type: 'rclone',
            host: config.value.rcloneHost,
            username: config.value.rcloneUserName,
            password: config.value.rclonePassword
        }).then(res => {
            ElMessage.success(res.message)
        }).finally(() => {
            rcloneTestLoading.value = false
        })
        return
    }
}
</script>

<style scoped>
.config-dialog {
    border-radius: 12px;
}

.config-dialog :deep(.el-dialog__header) {
    padding: 1.5rem 1.5rem 1rem;
    border-bottom: 1px solid var(--el-border-color-lighter);
}

.config-dialog :deep(.el-dialog__body) {
    padding: 1.5rem;
    max-height: 70vh;
    overflow: hidden;
}

.config-dialog :deep(.el-dialog__footer) {
    padding: 1rem 1.5rem 1.5rem;
    border-top: 1px solid var(--el-border-color-lighter);
}

.config-dialog :deep(.el-tabs__content) {
    padding-top: 1rem;
    max-height: 50vh;
    overflow: hidden;
}

.config-dialog :deep(.el-scrollbar__view) {
    padding-right: 1rem;
}

.config-dialog :deep(.el-form-item) {
    margin-bottom: 1.25rem;
}

.config-dialog :deep(.el-form-item__label) {
    font-weight: 500;
}

/* 响应式标签宽度 */
.config-dialog :deep(.el-form--label-right .el-form-item__label) {
    text-align: right;
}

/* 移动端适配 */
@media (max-width: 768px) {
    .config-dialog :deep(.el-dialog__header) {
        padding: 1rem 1rem 0.75rem;
    }

    .config-dialog :deep(.el-dialog__body) {
        padding: 1rem;
        max-height: 75vh;
    }

    .config-dialog :deep(.el-dialog__footer) {
        padding: 0.75rem 1rem 1rem;
    }

    .config-dialog :deep(.el-tabs__content) {
        max-height: 55vh;
    }

    .config-dialog :deep(.el-form) {
        width: 100%;
    }

    .config-dialog :deep(.el-form--label-right .el-form-item__label) {
        width: 120px !important;
        text-align: left;
        font-size: 14px;
    }

    .config-dialog :deep(.el-form-item) {
        margin-bottom: 1rem;
        flex-direction: column;
        align-items: stretch;
    }

    .config-dialog :deep(.el-form-item__content) {
        margin-left: 0 !important;
        margin-top: 0.5rem;
    }

    .config-dialog :deep(.el-input) {
        width: 100%;
    }

    .config-dialog :deep(.el-button) {
        font-size: 14px;
        padding: 8px 16px;
    }
}

@media (max-width: 480px) {
    .config-dialog :deep(.el-dialog__header) {
        padding: 0.75rem 0.75rem 0.5rem;
    }

    .config-dialog :deep(.el-dialog__body) {
        padding: 0.75rem;
        max-height: 80vh;
    }

    .config-dialog :deep(.el-dialog__footer) {
        padding: 0.5rem 0.75rem 0.75rem;
    }

    .config-dialog :deep(.el-tabs__content) {
        max-height: 60vh;
    }

    .config-dialog :deep(.el-form--label-right .el-form-item__label) {
        width: 100px !important;
        font-size: 13px;
    }

    .config-dialog :deep(.el-scrollbar__view) {
        padding-right: 0.5rem;
    }
}

/* 平板适配 */
@media (min-width: 769px) and (max-width: 1024px) {
    .config-dialog :deep(.el-form--label-right .el-form-item__label) {
        width: 160px !important;
    }

    .config-dialog :deep(.el-dialog__body) {
        max-height: 65vh;
    }

    .config-dialog :deep(.el-tabs__content) {
        max-height: 45vh;
    }
}
</style>