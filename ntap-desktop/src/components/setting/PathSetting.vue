<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { homeDir } from '@tauri-apps/api/path';
import { AppConfig } from '../../types/common';
import { useToast } from "primevue/usetoast";
const toast = useToast();

const configDirPath = ref('');
const appConfig: AppConfig = reactive(new AppConfig());

const selectConfigDir = async () => {
    // Open a selection dialog for directories
    const selectedDir = await open({
    directory: true,
    multiple: false,
    defaultPath: await homeDir(),
    });
    if (selectedDir === null) {
        // user cancelled the selection
    } else {
        if (Array.isArray(selectedDir)) {
            if (selectedDir.length > 0) {
                configDirPath.value = selectedDir[0];
            }
        } else {
            configDirPath.value = selectedDir;
        }
    }
}

const getAppConfig = async () => {
    invoke<AppConfig>('get_app_config').then((res) => {
        appConfig.logging.level = res.logging.level;
        appConfig.logging.file_path = res.logging.file_path;
        appConfig.network.interfaces = res.network.interfaces;
        appConfig.network.reverse_dns = res.network.reverse_dns;
        appConfig.network.entry_ttl = res.network.entry_ttl;
        appConfig.display.top_remote_hosts = res.display.top_remote_hosts;
        appConfig.display.connection_count = res.display.connection_count;
        appConfig.display.tick_rate = res.display.tick_rate;
        appConfig.display.show_bandwidth = res.display.show_bandwidth;
        appConfig.privacy.hide_private_ip_info = res.privacy.hide_private_ip_info;
        appConfig.privacy.hide_public_ip_info = res.privacy.hide_public_ip_info;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const saveAppConfig = async () => {
    invoke('save_app_config', {config: appConfig}).then(() => {
        toast.add({ severity: 'success', summary: 'Saved', detail: 'File setting saved', life: 2000 });
    }).catch((err) => {
        console.log(err);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to save file setting', life: 2000 });
    }).finally(() => {
        
    });
}

const getConfigDirPath = async () => {
    await invoke<string>('get_config_dir').then((res) => {
        configDirPath.value = res;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

onMounted(() => {
    getAppConfig();
    getConfigDirPath();
});

onUnmounted(() => {

});

</script>

<style scoped>
.bg-transparent {
    background-color: transparent;
}
</style>

<template>
<Toast />
<Card class="bg-transparent">
    <template #title>Path Setting</template>
    <template #content>
        <div class="flex flex-column gap-2">
            <label for="config-dir-path" class="font-bold block">Config Directory</label>
            <InputGroup>
                <InputText disabled id="config-dir-path" v-model="configDirPath" aria-describedby="config-dir-path-help" />
                <Button disabled icon="pi pi-folder" @click="selectConfigDir" />
            </InputGroup>
            <small id="config-dir-path-help">Config Directory</small>
        </div>
    </template>
    <template #footer>
        <div class="flex gap-3 mt-1">
            <Button disabled type="button" label="Cancel" severity="secondary" @click="getAppConfig" ></Button>
            <Button disabled type="button" label="Save" @click="saveAppConfig"></Button>
        </div>
    </template>
</Card>
</template>
