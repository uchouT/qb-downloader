<template>
  <Torrent ref="torrent" @load="handleTaskAdded"></Torrent>
  <Config ref="config"></Config>
  <div style="display: flex; flex-direction: column; height: 100%;">
    <header class="app-header" style="position: sticky; top: 0; z-index: 100;">
      <div class="header-content flex-center" style="padding: 1rem 2rem; justify-content: space-between;">
        <div class="app-logo" style="display: flex; align-items: center; gap: 0.05rem; flex: 1;">
          <img src="../../public/icon.png" class="app-icon" style=" width: 32px; height: 32px; flex-shrink: 0;" />
          <h1 class="app-title" style="margin: 0; font-size: 1.5rem; font-weight: 700;">b-downloader</h1>
        </div>
        <nav class="header-actions" style="">
          <el-button class="action-button add-button" type="primary" @click="addTask">
            <el-icon>
              <Plus />
            </el-icon>
            <span v-if="isNotMobile()">添加任务</span>
          </el-button>
          <el-button class="action-button config-button" @click="config?.show()">
            <el-icon>
              <Setting />
            </el-icon>
            <span v-if="isNotMobile()">设置</span>
          </el-button>
        </nav>
      </div>
    </header>
    <main
      style="width: 100%;max-width: 100%; height:100%; padding: 2rem; overflow: hidden; display: flex; flex-direction: column;">
      <List ref="taskList" />
    </main>

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
.app-header {
  border-bottom: 1px solid var(--el-border-color-light);
}


.app-title {
  background: linear-gradient(135deg, var(--el-color-primary), var(--el-color-primary-light-3));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.action-button:hover {
  transform: translateY(-1px);
}

/* 退出登录浮动按钮 */
.logout-button {
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  width: 3.5rem;
  height: 3.5rem;
  z-index: 999;

  transition: all 0.3s ease;
}

.logout-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(245, 108, 108, 0.4);
}
</style>