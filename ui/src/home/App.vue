<template>
  <Torrent ref="torrent"></Torrent>
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
        </div>
      </div>
    </div>
    <div style="flex: 1; overflow: hidden; padding: 16px;">
      <el-empty description="当前任务为空" />
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { useWindowSize } from "@vueuse/core";
import { Fold, Plus, Setting, Tickets } from "@element-plus/icons-vue"
import Torrent from "./Torrent.vue";
// 响应式数据
const { width, height } = useWindowSize();
const torrent = ref()
const task = ref()




// 方法定义
const elIconClass = () => {
  return isNotMobile() ? 'el-icon--left' : '';
}

const isNotMobile = () => {
  return width.value > 800;
}

// 事件处理函数
const handleAddSubscription = () => {
  console.log('添加订阅')
  // 这里添加具体的添加订阅逻辑
}

const handleAddCollection = () => {
  console.log('添加合集')
  // 这里添加具体的添加合集逻辑
}
</script>

<style>
@media (min-width: 1000px) {
  .auto {
    display: flex;
  }
}
</style>