<template>
    <el-dialog v-model="dialogVisible" title="设置" width="50%">
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
                            <el-form-item label="仅限内网 IP">
                                <el-switch v-model="config.onlyInnerIP" />
                            </el-form-item>
                            <el-form-item label="禁止多端登录">
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
                            <el-form-item label="Alist Host">
                                <el-input v-model="config.alistHost" placeholder="http://localhost:5244" />
                            </el-form-item>
                            <el-form-item label="Alist Token">
                                <el-input v-model="config.alistToken" type="password" placeholder="alist token" />
                            </el-form-item>
                            <el-form-item label="Rclone Host">
                                <el-input v-model="config.rcloneHost" placeholder="http://localhost:5572" />
                            </el-form-item>
                            <el-form-item label="Rclone Username">
                                <el-input v-model="config.rcloneUserName" placeholder="admin" />
                            </el-form-item>
                            <el-form-item label="Rclone Password">
                                <el-input v-model="config.rclonePassword" type="password" placeholder="secret" />
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
import { ref } from 'vue';
import api from '../api';
import CryptoJS from "crypto-js";
import { ElMessage } from 'element-plus';
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
const dialogVisible = ref(false)
const activeName = ref('first')
const config = ref({
    qbHost: "",
    qbUsername: "",
    qbPassword: '',
    alistHost: '',
    alistToken: '',
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
</script>