<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { AppConfig } from '../../types/common';
import { NetworkInterface } from '../../types/network';
import { useToast } from "primevue/usetoast";
const toast = useToast();

const selectedInterfaces = ref<NetworkInterface[]>([]);
const interfaces = ref<NetworkInterface[]>([]);

const reverseDns = ref('No');
const reverseDnsOptions = ref(['Yes', 'No']);

const appConfig: AppConfig = reactive(new AppConfig());

const selectInterfaces = (interfaceNames: string[]) => {
    selectedInterfaces.value = [];
    if (interfaceNames.length === 0) {
        // Select all interfaces
        selectedInterfaces.value = interfaces.value;
        return;
    }
    interfaceNames.forEach((name) => {
        const iface = interfaces.value.find((i) => i.name === name);
        if (iface) {
            selectedInterfaces.value.push(iface);
        }
    });
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

        reverseDns.value = appConfig.network.reverse_dns ? 'Yes' : 'No';

    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const getInterfaces = async () => {
    await invoke<NetworkInterface[]>('get_interfaces').then((res) => {
        interfaces.value = res;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const saveAppConfig = async () => {
    // Convert selected interfaces to interface names
    if (selectedInterfaces.value.length === interfaces.value.length) {
        // All interfaces are selected
        appConfig.network.interfaces = [];
    } else {
        appConfig.network.interfaces = selectedInterfaces.value.map((i) => i.name);
    }
    await invoke('save_app_config', {config: appConfig}).then(() => {
        toast.add({ severity: 'success', summary: 'Saved', detail: 'Network setting saved', life: 2000 });
    }).catch((err) => {
        console.log(err);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to save network setting', life: 2000 });
    }).finally(() => {
        
    });
}

const initSettings = async () => {
    await getInterfaces();
    await getAppConfig();
    selectInterfaces(appConfig.network.interfaces);
}

const onReverseDnsChange = () => {
    appConfig.network.reverse_dns = reverseDns.value === 'Yes';
}

onMounted(() => {
    initSettings();
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
    <template #title>Network Setting</template>
    <template #content>
        <div class="flex flex-column gap-2 mb-2">
            <label for="entry-ttl" class="font-bold block"> Entry TTL (milliseconds) </label>
            <InputNumber style="max-width: 25%;" v-model="appConfig.network.entry_ttl" inputId="entry-ttl" mode="decimal" showButtons :min="30000" :max="3600000" aria-describedby="entry-ttl-help" />
            <small id="entry-ttl-help"> Set the Time-To-Live for each network entry in milliseconds </small>
        </div>
        <div class="flex flex-column gap-2 mb-2">
            <label for="interface-names" class="font-bold block"> Interfaces </label>
            <MultiSelect v-model="selectedInterfaces" :options="interfaces" optionLabel="name" placeholder="Select interfaces" :maxSelectedLabels="3" class="w-full md:w-20rem" aria-describedby="interface-names-help" />
            <small id="interface-names-help"> Select which network interfaces to monitor </small>
        </div>
        <div class="flex flex-column gap-2">
            <label for="reverse-dns" class="font-bold block"> Reverse DNS </label>
            <SelectButton v-model="reverseDns" :options="reverseDnsOptions" aria-labelledby="private-ip-visible" @change="onReverseDnsChange" />
            <small id="reverse-dns-help"> Toggle the use of Reverse DNS lookups </small>
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
