<script setup lang="ts">
import { reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/shell';
import { AppInfo } from '../types/common';

const emit = defineEmits(["closeAboutDialog"]);

const props = defineProps({
    visible: {
    type: Boolean,
    },
});

const closeAboutDialog = () => {
    emit("closeAboutDialog");
};

const onVisibleUpdate = () => {
    emit("closeAboutDialog");
};

const aboutApp = reactive({
    name: 'Ntap Desktop',
    version: 'v0.1.0',
    release_date: '2024-05-02',
    author: 'shellrow <shellrow@foctet.com>',
    description: 'Cross-platform Network Utilization Statistics tool',
    repository: 'https://github.com/shellrow/ntap',
});

const getAppInfo = async () => {
    invoke<AppInfo>('get_app_info').then((res) => {
        //aboutApp.name = res.name;
        aboutApp.description = res.description;
        aboutApp.version = `v${res.version}`;
        aboutApp.release_date = res.release_date;
        aboutApp.repository = res.repository;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const openRepository = () => {
    open(aboutApp.repository);
}

const checkUpdate = () => {
    open("https://github.com/shellrow/ntap/releases");
}

onMounted(() => {
    getAppInfo();
});

onUnmounted(() => {

});

</script>

<style scoped>

</style>

<template>
<Dialog :visible="props.visible" @update:visible="onVisibleUpdate" modal header="About" :style="{width: '45vw'}" :breakpoints="{'960px': '75vw', '640px': '100vw'}">
    <template #header>
        <div class="inline-flex align-items-center justify-content-center gap-2">
            <span class="font-bold white-space-nowrap">About</span>
        </div>
    </template>
    <div class="flex flex-column align-items-center w-full gap-3 border-bottom-1 surface-border">
        <Avatar image="ntap.svg" shape="circle" size="large" />
        <span class="text-xl text-900 font-bold white-space-nowrap">{{ aboutApp.name }}</span>
        <span>{{ aboutApp.description }}</span>
        <div class="flex align-items-center gap-3 mb-2">
            <label for="version" class="font-semibold">Version</label>
            <Button :label="aboutApp.version" link @click="openRepository" />
        </div>
        <div class="flex align-items-center gap-3 mb-2">
            <label for="release_date" class="font-semibold">Release Date</label>
            <span class="text-900 font-medium">{{ aboutApp.release_date }}</span>
        </div>
        <Button class="mb-2" label="Check Update" outlined severity="secondary" @click="checkUpdate" size="small" />
    </div>
    <template #footer>
        <Button label="Close" outlined @click="closeAboutDialog" autofocus />
    </template>
</Dialog>
</template>
