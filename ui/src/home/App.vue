<template>
  <Torrent ref="torrent" @load="handleTaskAdded"></Torrent>
  <Config ref="config"></Config>
  <div class="app-container">
    <header class="app-header">
      <div class="header-content">
        <div class="app-logo">
          <h1 class="app-title">qb-downloader</h1>
        </div>
        <nav class="header-actions">
          <el-button 
            class="action-button add-button" 
            type="primary" 
            @click="torrent?.show()"
            :size="isNotMobile() ? 'default' : 'small'">
            <el-icon>
              <Plus />
            </el-icon>
            <span v-if="isNotMobile()">添加任务</span>
          </el-button>
          <el-button 
            class="action-button config-button" 
            @click="config?.show()"
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
  </div>
</template>

<script setup>
import { ref } from "vue";
import { useWindowSize } from "@vueuse/core";
import { Fold, Plus, Setting, Tickets } from "@element-plus/icons-vue"
import Torrent from "./Torrent.vue";
import Config from "./Config.vue";
import List from "./List.vue";

// 响应式数据
const { width, height } = useWindowSize();
const torrent = ref()
const config = ref()
const taskList = ref()

// 方法定义
const elIconClass = () => {
  return isNotMobile() ? 'el-icon--left' : '';
}

const isNotMobile = () => {
  return width.value > 768;
}

const isMobile = () => {
  return width.value <= 768;
}

const isTablet = () => {
  return width.value > 768 && width.value <= 1024;
}

const isDesktop = () => {
  return width.value > 1024;
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