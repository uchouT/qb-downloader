<template>
  <Torrent ref="torrent" @load="handleTaskAdded"></Torrent>
  <Config ref="config"></Config>
  <div style="height: 100%; display: flex; flex-direction: column;">
    <div id="header">
      <div style="margin: 10px;" class="auto">
        <div style="height: 8px; width: 8px;"></div>
      </div>
      <div style="margin: 10px; display: flex; justify-content: flex-end;">
        <div style="margin: 0 4px;">
          <el-button bg text type="primary" @click="torrent?.show()">
            <el-icon :class="elIconClass()">
              <Plus />
            </el-icon>
          </el-button>
          <el-button bg text type="primary" @click="config?.show()">
            <el-icon :class="elIconClass()">
              <Setting />
            </el-icon>
          </el-button>
        </div>
      </div>
    </div>
    <div style="flex: 1; overflow: hidden; padding: 16px;">
      <List ref="taskList" />
    </div>
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
  return width.value > 800;
}

// 当任务添加成功时刷新任务列表
const handleTaskAdded = () => {
  taskList.value?.refreshTasks()
}
</script>

<style>
@media (min-width: 1000px) {
  .auto {
    display: flex;
  }
}
</style>