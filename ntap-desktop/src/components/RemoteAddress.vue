<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { HostDisplayInfo } from '../types/network';
import { KVItem } from '../types/common';
import { WindowUtil } from '../util/window';
import { setRoutine } from '../util/routine';
import { SelectButtonChangeEvent } from 'primevue/selectbutton';

const windowUtil = new WindowUtil();
const autoUpdate = ref(true);
const updateType = ref('Auto');
const updateTypes = ref(
    [
        {value: 'Auto', icon: 'pi pi-play'},
        {value: 'Manual', icon: 'pi pi-pause'}
    ]
);
const trafficDisplayType = ref('Total');
const trafficDisplayTypes = ref(['Total', 'Bandwidth']);

const tableData = ref<HostDisplayInfo[]>([]);
const isLoading = ref(false);

const selectedHostKv = ref<KVItem[]>([]);

const selectedHost = ref<HostDisplayInfo>();

const dialogVisible = ref(false);

const onRowSelect = (event: any) => {
    let host: HostDisplayInfo = event.data;
    // set selectedHostKv. order is original order.
    selectedHostKv.value = [];
    selectedHostKv.value.push({key: 'IP Address', value: host.ip_addr});
    selectedHostKv.value.push({key: 'Host Name', value: host.host_name});
    selectedHostKv.value.push({key: 'Packet Sent', value: host.traffic.packet_sent.toString()});
    selectedHostKv.value.push({key: 'Packet Received', value: host.traffic.packet_received.toString()});
    selectedHostKv.value.push({key: 'Bytes Sent', value: host.traffic.bytes_sent.toString()});
    selectedHostKv.value.push({key: 'Bytes Received', value: host.traffic.bytes_received.toString()});
    selectedHostKv.value.push({key: 'Country Code', value: host.country_code});
    selectedHostKv.value.push({key: 'Country Name', value: host.country_name});
    selectedHostKv.value.push({key: 'ASN', value: host.asn.toString()});
    selectedHostKv.value.push({key: 'AS Name', value: host.as_name});
    dialogVisible.value = true;
};

const onRowUnselect = (_event: any) => {
    dialogVisible.value = false;
}

const GetRemoteHosts = async() => {
    isLoading.value = true;
    const remoteHosts: HostDisplayInfo[] = await invoke('get_remote_hosts');
    tableData.value = remoteHosts;
    isLoading.value = false;
}

const routine = setRoutine({
  interval: 1000,
  callback: () => { 
        if (autoUpdate.value) {
            GetRemoteHosts(); 
        }
    }
});

const onUpdateTypeChange = (event: SelectButtonChangeEvent) => {
    if (event.value === 'Auto') {
        autoUpdate.value = true;
    } else {
        autoUpdate.value = false;
    }
}

onMounted(() => {
    windowUtil.mount();
    GetRemoteHosts();
    routine.start();
});

onUnmounted(() => {
    windowUtil.unmount();
    routine.stop();
});

</script>

<style scoped>
.p-card, .p-card-title, .p-card-content {
    background-color: var(--surface-ground);
}
</style>

<template>
    <Card class="flex-auto">
        <template #title> 
            <div class="flex justify-content-between">
                <div class="flex">
                    
                </div>
                <div class="flex">
                    <SelectButton class="mr-2" v-model="trafficDisplayType" :options="trafficDisplayTypes" aria-labelledby="traffic-display-type" />
                    <SelectButton class="mr-2" v-model="updateType" :options="updateTypes" optionValue="value" optionLabel="value" dataKey="value" @change="onUpdateTypeChange" aria-labelledby="update-type">
                        <template #option="slotProps">
                            <i :class="slotProps.option.icon"></i>
                            <span>{{slotProps.option.value}}</span>
                        </template>
                    </SelectButton>
                    <Button type="button" icon="pi pi-refresh" outlined :loading="isLoading" @click="GetRemoteHosts" :disabled="autoUpdate" />
                </div>
            </div>
        </template>
        <template #content>
            <DataTable :value="tableData" v-model:selection="selectedHost" :virtualScrollerOptions="{ itemSize: 20 }" selectionMode="single" dataKey="ip_addr" @rowSelect="onRowSelect" @rowUnselect="onRowUnselect" size="small" scrollable :scrollHeight="(windowUtil.windowSize.innerHeight-200).toString() + 'px'" tableStyle="min-width: 30rem">
                <Column field="ip_addr" header="IP Address" sortable></Column>
                <!-- <Column field="host_name" header="Host Name" sortable></Column> -->
                <div v-if="trafficDisplayType == 'Bandwidth'">
                    <Column field="traffic.formatted_egress_packets_per_sec" header="Packet Sent" sortable></Column>
                    <Column field="traffic.formatted_ingress_packets_per_sec" header="Packet Recv" sortable></Column>
                    <Column field="traffic.formatted_egress_bytes_per_sec" header="Bytes Sent" sortable></Column>
                    <Column field="traffic.formatted_ingress_bytes_per_sec" header="Bytes Recv" sortable></Column>
                </div>
                <div v-else>
                    <Column field="traffic.packet_sent" header="Packet Sent" sortable></Column>
                    <Column field="traffic.packet_received" header="Packet Recv" sortable></Column>
                    <Column field="traffic.formatted_sent_bytes" header="Bytes Sent" sortable></Column>
                    <Column field="traffic.formatted_received_bytes" header="Bytes Recv" sortable></Column>
                </div>
                <Column field="country_code" header="Country" sortable></Column>
                <Column field="asn" header="ASN" sortable></Column>
                <Column field="as_name" header="AS Name" sortable></Column>
            </DataTable>
        </template>
    </Card>
    <Dialog v-model:visible="dialogVisible" :modal="false" :closable="true" header="Detail" :showHeader="true" :breakpoints="{'960px': '75vw', '640px': '100vw'}" :style="{width: '45vw'}">
        <DataTable :value="selectedHostKv"  scrollable scrollHeight="70vh" tableStyle="min-width: 50rem">
                <Column field="key" header="" ></Column>
                <Column field="value" header="" ></Column>
            </DataTable>
        <template #footer>
            <div class="flex border-top-1 pt-5 surface-border justify-content-end align-items-center">
                <Button @click="dialogVisible = false" icon="pi pi-check" label="OK" class="m-0"></Button>
            </div>
        </template>
    </Dialog>
</template>
