<template>
  <div class="login-container" v-if="!authorization">
    <div class="login-background">
      <div class="background-overlay"></div>
    </div>
    
    <div class="login-content">
      <div class="login-card">
        <div class="login-header">
          <div style="text-align: center;">
          <img src="../public/icon.png" height="100" width="100" alt="icon.png"/>
        </div>
        </div>
        
        <el-form class="login-form" @keyup.enter="login" @submit="login">
          <el-form-item>
            <el-input 
              v-model="user.username" 
              placeholder="用户名" 
              size="large"
              class="login-input">
              <template #prefix>
                <el-icon class="input-icon">
                  <User/>
                </el-icon>
              </template>
            </el-input>
          </el-form-item>
          
          <el-form-item>
            <el-input 
              v-model="user.password" 
              show-password
              placeholder="密码" 
              size="large"
              class="login-input">
              <template #prefix>
                <el-icon class="input-icon">
                  <Key/>
                </el-icon>
              </template>
            </el-input>
          </el-form-item>
          
          <div class="login-options">
            <el-checkbox v-model="rememberThePassword.remember" class="remember-checkbox">
              记住密码
            </el-checkbox>
          </div>
          
          <el-button 
            @click="login" 
            :loading="loading" 
            type="primary"
            size="large"
            class="login-button">
            <el-icon v-if="!loading">
              <Right />
            </el-icon>
            {{ loading ? '登录中...' : '登录' }}
          </el-button>
        </el-form>
      </div>
      
      <div class="login-footer">
        <a href="https://github.com/uchouT/qb-downloader" target="_blank" class="github-link">
          <el-icon><Link /></el-icon>
          GitHub
        </a>
      </div>
    </div>
  </div>
  <App v-else></App>
</template>

<script setup>

import {onMounted, ref} from "vue";
import CryptoJS from "crypto-js"
import App from "./home/App.vue";
import api from "./api.js";
import {useDark, useLocalStorage} from '@vueuse/core'
import {Key, User, Right, Link} from "@element-plus/icons-vue";

let loading = ref(false)

let authorization = ref("")
let user = ref({
  'username': '',
  'password': ''
})

authorization.value = localStorage.getItem("authorization")
if (authorization.value) {
  window.authorization = authorization.value
}

let rememberThePassword = useLocalStorage('rememberThePassword', {
  remember: false,
  username: '',
  password: ''
})

let login = () => {
  loading.value = true
  user.value.password = user.value.password.trim()
  user.value.username = user.value.username.trim()
  let my_user = JSON.parse(JSON.stringify(user.value))
  my_user.password = my_user.password ? CryptoJS.MD5(my_user.password).toString() : '';
  api.post('api/login', my_user)
      .then(res => {
        localStorage.setItem("authorization", res.data)
        window.authorization = res.data
        authorization.value = res.data

        // 记住密码
        if (rememberThePassword.value.remember) {
          rememberThePassword.value.username = user.value.username
          rememberThePassword.value.password = user.value.password
        } else {
          rememberThePassword.value.username = ''
          rememberThePassword.value.password = ''
        }
      })
      .finally(() => {
        loading.value = false
      })
}

let test = () => {
  if (window.authorization) {
    return
  }
  fetch('api/test', {
    'headers': {
      'Authorization': window.authorization
    }
  })
      .then(res => res.json())
      .then(res => {
        if (res.code === 200) {
          localStorage.setItem("authorization", '1')
          window.authorization = '1'
          authorization.value = '1'
          return
        }
        localStorage.removeItem("authorization")
        window.authorization = ''
        authorization.value = ''
      })
}

useDark()

onMounted(() => {
  test()
  let {remember, username, password} = rememberThePassword.value;
  if (remember && username && password) {
    user.value.username = username
    user.value.password = password
  }
})


// document.documentElement 是全局变量时
const el = document.documentElement
// const el = document.getElementById('xxx')

// 获取 css 变量
getComputedStyle(el).getPropertyValue(`--el-color-primary`)

// 设置 css 变量
el.style.setProperty('--el-color-primary', useLocalStorage('--el-color-primary', '#409eff').value)

</script>

<style scoped>
.login-container {
  position: relative;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.login-background {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, 
    var(--el-color-primary-light-9) 0%, 
    var(--el-color-primary-light-8) 25%,
    var(--el-color-primary-light-7) 50%,
    var(--el-color-primary-light-6) 75%,
    var(--el-color-primary-light-5) 100%
  );
  z-index: 1;
}

.background-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: radial-gradient(circle at 30% 40%, rgba(255, 255, 255, 0.1) 0%, transparent 50%),
              radial-gradient(circle at 80% 20%, rgba(255, 255, 255, 0.05) 0%, transparent 50%),
              radial-gradient(circle at 40% 80%, rgba(255, 255, 255, 0.08) 0%, transparent 50%);
}

.login-content {
  position: relative;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 2rem;
  width: 100%;
  max-width: 400px;
}

.login-card {
  background: var(--el-bg-color);
  border-radius: 16px;
  padding: 3rem 2.5rem;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
  border: 1px solid var(--el-border-color-lighter);
  width: 100%;
  max-width: 400px;
  transition: all 0.3s ease;
}

.login-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 25px 70px rgba(0, 0, 0, 0.15);
}

.login-header {
  text-align: center;
  margin-bottom: 2.5rem;
}

.app-subtitle {
  margin: 0;
  font-size: 0.875rem;
  color: var(--el-text-color-secondary);
  line-height: 1.5;
}

.login-form {
  width: 100%;
}

.login-form .el-form-item {
  margin-bottom: 1.5rem;
}

.login-input {
  width: 100%;
}

.login-input :deep(.el-input__wrapper) {
  border-radius: 12px;
  padding: 12px 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
}

.login-input :deep(.el-input__wrapper):hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.login-input :deep(.el-input__wrapper.is-focus) {
  box-shadow: 0 4px 16px rgba(var(--el-color-primary-rgb), 0.2);
}

.input-icon {
  color: var(--el-color-primary);
  margin-right: 8px;
}

.login-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.remember-checkbox {
  color: var(--el-text-color-secondary);
}

.login-button {
  width: 100%;
  height: 48px;
  border-radius: 12px;
  font-size: 1rem;
  font-weight: 500;
  background: linear-gradient(135deg, var(--el-color-primary), var(--el-color-primary-light-3));
  border: none;
  transition: all 0.3s ease;
}

.login-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 20px rgba(var(--el-color-primary-rgb), 0.3);
}

.login-button .el-icon {
  margin-right: 8px;
}

.login-footer {
  margin-top: 2rem;
  text-align: center;
}

.github-link {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
  color: var(--el-text-color-secondary);
  text-decoration: none;
  font-size: 0.875rem;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border: 1px solid var(--el-border-color-lighter);
  transition: all 0.3s ease;
}

.github-link:hover {
  color: var(--el-color-primary);
  background: var(--el-bg-color);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .login-content {
    padding: 1rem;
  }
  
  .login-card {
    padding: 2rem 1.5rem;
    border-radius: 12px;
    margin: 1rem 0;
  }
  
  .app-subtitle {
    font-size: 0.8rem;
  }
  
  .login-header {
    margin-bottom: 2rem;
  }
  
  .login-form .el-form-item {
    margin-bottom: 1.25rem;
  }
  
  .login-options {
    margin-bottom: 1.5rem;
  }
  
  .login-button {
    height: 44px;
    font-size: 0.9rem;
  }
}

@media (max-width: 480px) {
  .login-content {
    padding: 0.75rem;
  }
  
  .login-card {
    padding: 1.5rem 1.25rem;
    border-radius: 8px;
  }
  
  
  .app-subtitle {
    font-size: 0.75rem;
  }
  
  .login-input :deep(.el-input__wrapper) {
    padding: 10px 12px;
  }
  
  .remember-checkbox :deep(.el-checkbox__label) {
    font-size: 0.8rem;
  }
  
  .github-link {
    font-size: 0.8rem;
    padding: 0.4rem 0.8rem;
  }
}

@media (max-width: 360px) {
  .login-card {
    padding: 1.25rem 1rem;
  }
  

  .login-button {
    height: 42px;
    font-size: 0.85rem;
  }
}

/* 大屏幕优化 */
@media (min-width: 1200px) {
  .login-card {
    padding: 3.5rem 3rem;
    max-width: 450px;
  }
  
  .app-subtitle {
    font-size: 1rem;
  }
  
  .login-button {
    height: 52px;
    font-size: 1.1rem;
  }
}

/* 横屏模式适配 */
@media (max-height: 600px) and (orientation: landscape) {
  .login-content {
    min-height: auto;
    justify-content: center;
    padding: 1rem;
  }
  
  .login-card {
    padding: 1.5rem 2rem;
  }
  
  .login-header {
    margin-bottom: 1.5rem;
  }
  
  .login-form .el-form-item {
    margin-bottom: 1rem;
  }
  
  .login-options {
    margin-bottom: 1.25rem;
  }
  
  .login-footer {
    margin-top: 1rem;
  }
}

/* 深色模式优化 */
@media (prefers-color-scheme: dark) {
  .login-background {
    background: linear-gradient(135deg, 
      #1a1a1a 0%, 
      #2a2a2a 25%,
      #1e1e1e 50%,
      #262626 75%,
      #1a1a1a 100%
    );
  }
  
  .background-overlay {
    background: radial-gradient(circle at 30% 40%, rgba(255, 255, 255, 0.02) 0%, transparent 50%),
                radial-gradient(circle at 80% 20%, rgba(255, 255, 255, 0.01) 0%, transparent 50%),
                radial-gradient(circle at 40% 80%, rgba(255, 255, 255, 0.015) 0%, transparent 50%);
  }
  
  .github-link {
    background: rgba(0, 0, 0, 0.3);
  }
}

/* 高对比度模式 */
@media (prefers-contrast: high) {
  .login-card {
    border: 2px solid var(--el-border-color);
  }
  
  .login-input :deep(.el-input__wrapper) {
    border: 1px solid var(--el-border-color);
  }
  
  .github-link {
    border: 2px solid var(--el-border-color);
  }
}

/* 减少动画模式 */
@media (prefers-reduced-motion: reduce) {
  .login-card,
  .login-button,
  .github-link,
  .login-input :deep(.el-input__wrapper) {
    transition: none;
  }
  
  .login-card:hover,
  .login-button:hover,
  .github-link:hover {
    transform: none;
  }
}
</style>
