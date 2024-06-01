<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { AppConfig } from '../../types/common';
import { useToast } from "primevue/usetoast";
const toast = useToast();

const privateIpInfoVisible = ref('Hide');
const privateIpInfoVisibleTypes = ref(['Show', 'Hide']);
const publicIpInfoVisible = ref('Hide');
const publicIpInfoVisibleTypes = ref(['Show', 'Hide']);

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
        if (res.privacy.hide_private_ip_info) {
            privateIpInfoVisible.value = 'Hide';
        } else {
            privateIpInfoVisible.value = 'Show';
        }
        if (res.privacy.hide_public_ip_info) {
            publicIpInfoVisible.value = 'Hide';
        } else {
            publicIpInfoVisible.value = 'Show';
        }
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const saveAppConfig = async () => {
    invoke('save_app_config', {config: appConfig}).then(() => {
        toast.add({ severity: 'success', summary: 'Saved', detail: 'Privacy setting saved', life: 2000 });
    }).catch((err) => {
        console.log(err);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to save privacy setting', life: 2000 });
    }).finally(() => {
        
    });
}

const onPrivateIpInfoVisibleChange = () => {
    appConfig.privacy.hide_private_ip_info = privateIpInfoVisible.value === 'Hide';
}

const onPublicIpInfoVisibleChange = () => {
    appConfig.privacy.hide_public_ip_info = publicIpInfoVisible.value === 'Hide';
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
    <template #title>Privacy Setting</template>
    <template #content>
        <div class="flex flex-column gap-2 mb-2">
            <label for="hide-private-ip-info" class="font-bold block"> Self-Private IP Info </label>
            <SelectButton v-model="privateIpInfoVisible" :options="privateIpInfoVisibleTypes" aria-labelledby="private-ip-visible" @change="onPrivateIpInfoVisibleChange" />
            <small id="hide-private-ip-info-help">Show/Hide self-private IP Info by default</small>
        </div>
        <div class="flex flex-column gap-2">
            <label for="hide-public-ip-info" class="font-bold block"> Self-Public IP Info </label>
            <SelectButton v-model="publicIpInfoVisible" :options="publicIpInfoVisibleTypes" aria-labelledby="public-ip-visible" @change="onPublicIpInfoVisibleChange" />
            <small id="hide-public-ip-info-help">Show/Hide self-public IP Info by default</small>
        </div>
    </template>
    <template #footer>
        <div class="flex gap-3 mt-1">
            <Button type="button" label="Cancel" severity="secondary"></Button>
            <Button type="button" label="Save" @click="saveAppConfig"></Button>
        </div>
    </template>
</Card>
</template>
