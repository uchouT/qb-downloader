<template>
    <el-dialog v-model="dialogVisible" append-to-body title="种子内容" :width="dialogWidth" v-loading="loading">
        <el-tree-v2 ref="tree" show-checkbox :default-checked-keys="defaultCheckedKeys"
            :default-expanded-keys="defaultExpandedKeys" node-key="id">
            <template #default="{ node }">
                <el-icon class="el-icon--left">
                    <Document v-if="node.isLeaf" />
                    <Folder v-else-if="!node.expanded" />
                    <FolderOpened v-else />
                </el-icon>
                <span>{{ node.label }}</span>
            </template></el-tree-v2>
        <div style="text-align: right;">
            <el-button type="primary" @click="save">保存</el-button>
        </div>
    </el-dialog>
</template>

<script setup>
import { ref, computed } from 'vue';
import { useWindowSize } from '@vueuse/core';
import { Folder, Document, FolderOpened } from '@element-plus/icons-vue';
import api from '../api';

const props = defineProps(['hash']);
const selectedFileIndex = defineModel('selectedFileIndex');
const dialogVisible = ref(false);
const loading = ref(false);
const defaultCheckedKeys = ref([-1]);
const defaultExpandedKeys = ref([-1]);
const { width } = useWindowSize();
const dialogWidth = computed(() => {
    if (width.value <= 768) {
        return '95%'
    } else if (width.value <= 1024) {
        return '70%'
    } else {
        return '50%'
    }
})

const tree = ref();
const save = () => {
    selectedFileIndex.value = tree.value.getCheckedKeys(true);
    dialogVisible.value = false;
}
const show = () => {
    dialogVisible.value = true;
    loading.value = true;
    api.get(`/api/torrent?hash=${props.hash}`)
        .then((res) => {
            tree.value.setData(res.data);
        }).finally(() => {
            loading.value = false;
        })
}
defineExpose({ show })
</script>