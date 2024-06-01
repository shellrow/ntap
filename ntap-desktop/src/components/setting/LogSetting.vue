<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { homeDir } from '@tauri-apps/api/path';
import { AppConfig } from '../../types/common';
import { useToast } from "primevue/usetoast";
const toast = useToast();

const appConfig: AppConfig = reactive(new AppConfig());

const selectedlogLevel = ref();

const logLevels = ref([
    { key: 'INFO', value: 'Info' },
    { key: 'WARN', value: 'Warning' },
    { key: 'ERROR', value: 'Error' },
]);

const selectLogFile = async () => {
    // Open a selection dialog for directories
    const selectedFile = await open({
    directory: false,
    multiple: false,
    defaultPath: await homeDir(),
    });
    if (selectedFile === null) {
        // user cancelled the selection
    } else {
        if (Array.isArray(selectedFile)) {
            if (selectedFile.length > 0) {
                appConfig.logging.file_path = selectedFile[0];
            }
        } else {
            appConfig.logging.file_path = selectedFile;
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

        selectedlogLevel.value = logLevels.value.find((item) => item.key === appConfig.logging.level);

    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const saveAppConfig = async () => {
    appConfig.logging.level = selectedlogLevel.value.key;
    appConfig.logging.file_path = appConfig.logging.file_path;
    invoke('save_app_config', {config: appConfig}).then(() => {
        toast.add({ severity: 'success', summary: 'Saved', detail: 'Log setting saved', life: 2000 });
    }).catch((err) => {
        console.log(err);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to save log setting', life: 2000 });
    }).finally(() => {
        
    });
}

onMounted(() => {
    getAppConfig();
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
    <template #title>Log Setting</template>
    <template #content>
        <div class="flex flex-column gap-2 mb-2">
            <label for="config-dir-path" class="font-bold block">Log Level</label>
            <Dropdown v-model="selectedlogLevel" :options="logLevels" optionLabel="value" placeholder="Select a Log level" class="w-full md:w-14rem" />
        </div>
        <div class="flex flex-column gap-2">
            <label for="config-dir-path" class="font-bold block">Log File</label>
            <InputGroup>
                <InputText id="config-dir-path" v-model="appConfig.logging.file_path" aria-describedby="config-dir-path-help" />
                <Button icon="pi pi-folder" @click="selectLogFile" />
            </InputGroup>
        </div>
    </template>
    <template #footer>
        <div class="flex gap-3 mt-1">
            <Button type="button" label="Cancel" severity="secondary" @click="getAppConfig" ></Button>
            <Button type="button" label="Save" @click="saveAppConfig"></Button>
        </div>
    </template>
</Card>
</template>
