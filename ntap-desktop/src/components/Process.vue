<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { KVItem } from '../types/common';
import { ProcessDisplayInfo } from '../types/network';
import { WindowUtil } from '../util/window';
import { setRoutine } from '../util/routine';
import { DataTableRowSelectEvent } from 'primevue/datatable';
import { SelectButtonChangeEvent } from 'primevue/selectbutton';

const tableData = ref<ProcessDisplayInfo[]>([]);
const selectedHostKv = ref<KVItem[]>([]);
const isLoading = ref(false);
const selectedHost = ref<any>();
const dialogVisible = ref(false);
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

const routine = setRoutine({
  interval: 1000,
  callback: () => { 
        if (autoUpdate.value) {
            GetProcessInfo(); 
        }
    }
});

const GetProcessInfo = async() => {
    isLoading.value = true;
    const result = await invoke<ProcessDisplayInfo[]>('get_process_info');
    tableData.value = result;
    isLoading.value = false;
}

const onRowSelect = (event: DataTableRowSelectEvent) => {
    const process_info: ProcessDisplayInfo = event.data;
    selectedHostKv.value = [];
    selectedHostKv.value.push({key: 'Process ID', value: process_info.pid.toString()});
    selectedHostKv.value.push({key: 'Process Name', value: process_info.name});
    selectedHostKv.value.push({key: 'Bytes Sent', value: process_info.traffic.bytes_sent.toString()});
    selectedHostKv.value.push({key: 'Bytes Received', value: process_info.traffic.bytes_received.toString()});
    selectedHostKv.value.push({key: 'Packets Sent', value: process_info.traffic.packet_sent.toString()});
    selectedHostKv.value.push({key: 'Packets Received', value: process_info.traffic.packet_received.toString()});
    dialogVisible.value = true;
};

const onRowUnselect = (_event: DataTableRowSelectEvent) => {
    dialogVisible.value = false;
}

const onUpdateTypeChange = (event: SelectButtonChangeEvent) => {
    if (event.value === 'Auto') {
        autoUpdate.value = true;
    } else {
        autoUpdate.value = false;
    }
}

onMounted(() => {
    windowUtil.mount();
    GetProcessInfo();
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
    <Card>
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
                    <Button type="button" icon="pi pi-refresh" outlined :loading="isLoading" @click="GetProcessInfo" :disabled="autoUpdate" />
                </div>
            </div>
        </template>
        <template #content>
            <DataTable :value="tableData" v-model:selection="selectedHost" :virtualScrollerOptions="{ itemSize: 20 }" selectionMode="single" dataKey="pid" @rowSelect="onRowSelect" @rowUnselect="onRowUnselect" size="small" scrollable :scrollHeight="(windowUtil.windowSize.innerHeight-200).toString() + 'px'" tableStyle="min-width: 30rem">
                <Column field="pid" header="Process ID" sortable></Column>
                <Column field="name" header="Process Name" sortable></Column>
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
