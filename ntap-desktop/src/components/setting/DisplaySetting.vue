<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { AppConfig } from '../../types/common';
import { useToast } from "primevue/usetoast";
const toast = useToast();

const trafficDisplayType = ref('Total');
const trafficDisplayTypes = ref(['Total', 'Bandwidth']);

const appConfig: AppConfig = reactive(new AppConfig());

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
        toast.add({ severity: 'success', summary: 'Saved', detail: 'Display setting saved', life: 2000 });
    }).catch((err) => {
        console.log(err);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to save display setting', life: 2000 });
    }).finally(() => {
        
    });
}

const onDisplayTypeChange = () => {
    appConfig.display.show_bandwidth = trafficDisplayType.value === 'Bandwidth';
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
    <template #title>Display Setting</template>
    <template #content>
        <div class="flex-auto mb-2">
            <label for="traffic-display-type" class="font-bold block"> Default Traffic Display Type </label>
            <SelectButton v-model="trafficDisplayType" :options="trafficDisplayTypes" aria-labelledby="traffic-display-type" @change="onDisplayTypeChange" />
        </div>
        <div class="flex flex-column gap-2 mb-2">
            <label for="top-remote-hosts" class="font-bold block"> Top Remote Hosts </label>
            <InputNumber style="max-width: 25%;" v-model="appConfig.display.top_remote_hosts" inputId="top-remote-hosts" mode="decimal" showButtons :min="1" :max="100" aria-describedby="top-remote-hosts-help" />
            <small id="top-remote-hosts-help"> Set the number of top remote hosts to display </small>
        </div>
        <div class="flex flex-column gap-2 mb-2">
            <label for="top-connections" class="font-bold block"> Top Connections </label>
            <InputNumber style="max-width: 25%;" v-model="appConfig.display.connection_count" inputId="top-connections" mode="decimal" showButtons :min="1" :max="100" aria-describedby="top-connections-help" />
            <small id="top-connections-help"> Set the number of top connections to display </small>
        </div>
        <div class="flex flex-column gap-2">
            <label for="tick-rate" class="font-bold block"> Tick Rate (milliseconds) </label>
            <InputNumber style="max-width: 25%;" v-model="appConfig.display.tick_rate" inputId="tick-rate" mode="decimal" showButtons :min="1000" :max="60000" aria-describedby="tick-rate-help" />
            <small id="tick-rate-help"> Set the refresh rate of the application in milliseconds </small>
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
