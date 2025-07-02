<template>
  <Torrent ref="torrent" @load="handleTaskAdded"></Torrent>
  <Config ref="config"></Config>
  <div class="app-container">
    <header class="app-header">
      <div class="header-content">
        <div class="app-logo">
          <img src="../../public/icon.png" class="app-icon" />
          <h1 class="app-title">b-downloader</h1>
        </div>
        <nav class="header-actions">
          <el-button class="action-button add-button" type="primary" @click="addTask"
            :size="isNotMobile() ? 'default' : 'small'">
            <el-icon>
              <Plus />
            </el-icon>
            <span v-if="isNotMobile()">添加任务</span>
          </el-button>
          <el-button class="action-button config-button" @click="config?.show()"
            :size="isNotMobile() ? 'default' : 'small'">
            <el-icon>
              <Setting />
            </el-icon>
            <span v-if="isNotMobile()">设置</span>
          </el-button>
        </nav>
      </div>
    </header>
    <main class="app-main">
      <div class="main-content">
        <List ref="taskList" />
      </div>
    </main>

    <!-- 浮动退出登录按钮 -->
    <el-button class="logout-button" @click="logout" type="danger">
      退出
    </el-button>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { useWindowSize } from "@vueuse/core";
import { ElMessage } from 'element-plus'
import { Plus, Setting, Tickets } from "@element-plus/icons-vue"
import Torrent from "./Torrent.vue";
import Config from "./Config.vue";
import List from "./List.vue";
import api from "../api";

// TODO: check
const checkBeforeAddTask = async () => {
  let uploaderOk
  let qbOk
  await api.get("api/test")
    .then(res => {
      uploaderOk = res['data'].uploaderOk;
      qbOk = res['data'].qbOk;
    })
  return uploaderOk && qbOk;
}


// 响应式数据
const { width, height } = useWindowSize();
const torrent = ref()
const config = ref()
const taskList = ref()

const addTask = async () => {
  const isReady = await checkBeforeAddTask();
  if (!isReady) {
    ElMessage.warning("qb 或者 uploader 未配置完成")
    return;
  }
  torrent?.value.show();
}

const isNotMobile = () => {
  return width.value > 768;
}

const logout = () => {
  localStorage.removeItem('authorization')
  location.reload()
}

// 当任务添加成功时刷新任务列表
const handleTaskAdded = () => {
  taskList.value?.refreshTasks()
}
</script>

<style scoped>
.app-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color-page);
}

.app-header {
  background: var(--el-bg-color);
  border-bottom: 1px solid var(--el-border-color-light);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-content {
  max-width: 100%;
  margin: 0 auto;
  padding: 1rem 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.app-logo {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 0.05rem;
}

.app-title {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--el-color-primary);
  background: linear-gradient(135deg, var(--el-color-primary), var(--el-color-primary-light-3));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.app-icon {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
}

.header-actions {
  display: flex;
  gap: 0.75rem;
  align-items: center;
}

.action-button {
  border-radius: 8px;
  transition: all 0.3s ease;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.action-button:hover {
  transform: translateY(-1px);
}

.add-button {
  background: linear-gradient(135deg, var(--el-color-primary), var(--el-color-primary-light-3));
  border: none;
}

.config-button {
  background: var(--el-fill-color-light);
  color: var(--el-text-color-primary);
  border: 1px solid var(--el-border-color);
}

.config-button:hover {
  background: var(--el-fill-color);
  border-color: var(--el-color-primary);
}

.app-main {
  flex: 1;
  overflow: hidden;
  display: flex;
  justify-content: center;
}

.main-content {
  width: 100%;
  max-width: 100%;
  padding: 2rem;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 退出登录浮动按钮 */
.logout-button {
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  width: 3.5rem;
  height: 3.5rem;
  z-index: 999;
  box-shadow: 0 4px 12px rgba(245, 108, 108, 0.3);
  border: none;
  transition: all 0.3s ease;
}

.logout-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(245, 108, 108, 0.4);
}

.logout-button:active {
  transform: translateY(0);
}

.logout-button .el-icon {
  font-size: 1.25rem;
}

/* 移动端适配 */
@media (max-width: 768px) {
  .header-content {
    padding: 0.75rem 1rem;
  }

  .app-title {
    font-size: 1.25rem;
  }

  .header-actions {
    gap: 0.5rem;
  }

  .action-button {
    padding: 0.5rem;
    min-width: auto;
  }

  .main-content {
    padding: 1rem;
  }

  .logout-button {
    bottom: 1.5rem;
    right: 1.5rem;
    width: 3rem;
    height: 3rem;
  }

  .logout-button .el-icon {
    font-size: 1.125rem;
  }
}

@media (max-width: 480px) {
  .header-content {
    padding: 0.5rem 0.75rem;
  }

  .app-title {
    font-size: 1.125rem;
  }

  .main-content {
    padding: 0.75rem;
  }

  .logout-button {
    bottom: 1rem;
    right: 1rem;
    width: 2.75rem;
    height: 2.75rem;
  }

  .logout-button .el-icon {
    font-size: 1rem;
  }
}

/* 平板适配 */
@media (min-width: 769px) and (max-width: 1024px) {
  .header-content {
    padding: 1rem 2rem;
  }

  .main-content {
    padding: 2rem;
  }
}

/* 桌面端适配 */
@media (min-width: 1025px) {
  .header-content {
    padding: 1.25rem 3rem;
  }

  .app-title {
    font-size: 1.75rem;
  }

  .main-content {
    padding: 2.5rem;
  }
}

/* 大屏适配 */
@media (min-width: 1440px) {
  .header-content {
    padding: 1.5rem 4rem;
  }

  .app-title {
    font-size: 2rem;
  }

  .main-content {
    padding: 3rem;
  }
}

/* 超大屏适配 */
@media (min-width: 1920px) {
  .header-content {
    padding: 1.75rem 6rem;
  }

  .app-title {
    font-size: 2.25rem;
  }

  .main-content {
    padding: 4rem;
  }
}
</style>