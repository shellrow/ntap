<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { KVItem, OptionItem } from '../types/common';
import { SocketDisplayInfo, SocketInfoOption } from '../types/network';
import { setRoutine } from '../util/routine';
import { WindowUtil } from '../util/window';
import { DataTableRowSelectEvent } from 'primevue/datatable';
import { SelectButtonChangeEvent } from 'primevue/selectbutton';

const tableData = ref<SocketDisplayInfo[]>([]);
const selectedHostKv = ref<KVItem[]>([]);
const selectedHost = ref<any>();
const selectedAddressFamily = ref<OptionItem[]>([]);
const selectedTransportProtocol = ref<OptionItem[]>([]);
const dialogVisible = ref(false);
const isLoading = ref(false);
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

const windowUtil = new WindowUtil();

const GetNetStat = async() => {
    isLoading.value = true;
    let options: SocketInfoOption = {
        address_family: [],
        transport_protocol: [],
    };
    if (selectedAddressFamily.value) {
        selectedAddressFamily.value.forEach((item) => {
            options.address_family?.push(item.id);
        });
    } else {
        options.address_family = ['IPv4', 'IPv6'];
    }
    if (selectedTransportProtocol.value) {
        selectedTransportProtocol.value.forEach((item) => {
            options.transport_protocol?.push(item.id);
        });
    } else {
        options.transport_protocol = ['TCP'];
    }
    const result = await invoke<SocketDisplayInfo[]>('get_netstat', {opt: options});
    tableData.value = result;
    isLoading.value = false;
}

const routine = setRoutine({
  interval: 1000,
  callback: () => { 
        if (autoUpdate.value) {
            GetNetStat(); 
        }
    }
});

const setDefaultOptions = () => {
    selectedAddressFamily.value = [
        { id: 'IPv4', name: ' IPv4' },
        { id: 'IPv6', name: ' IPv6' }
    ];
    selectedTransportProtocol.value = [
        { id: 'TCP', name: ' TCP' }
    ];
}

const address_families: OptionItem[] = [
    { id: 'IPv4', name: ' IPv4' },
    { id: 'IPv6', name: ' IPv6' }
];

const transport_protocols: OptionItem[] = [
    { id: 'TCP', name: ' TCP' },
    { id: 'UDP', name: ' UDP' }
];

const generateRowKey = (row: SocketDisplayInfo) => {
    const key = row.interface_name + ':' + row.local_port + '-' + row.remote_ip_addr + ':' + row.remote_port + '-' + row.protocol;
    return key;
}

const onRowSelect = (event: DataTableRowSelectEvent) => {
    dialogVisible.value = true;
    const socket_info: SocketDisplayInfo = event.data;
    selectedHostKv.value = [
        {
            key: 'IP Version',
            value: socket_info.ip_version,
        },
        {
            key: 'Interface Name',
            value: socket_info.interface_name || '',
        },
        {
            key: 'Local IP Address',
            value: socket_info.local_ip_addr || '',
        },
        {
            key: 'Local Port',
            value: socket_info.local_port.toString() || '',
        },
        {
            key: 'Remote IP Address',
            value: socket_info.remote_ip_addr || '',
        },
        {
            key: 'Remote Port',
            value: socket_info.remote_port?.toString() || '',
        },
        {
            key: 'Protocol',
            value: socket_info.protocol,
        },
        {
            key: 'Process ID',
            value: socket_info.process?.pid?.toString() || '',
        },
        {
            key: 'Process Name',
            value: socket_info.process?.name || '',
        },
    ];
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
    setDefaultOptions();
    GetNetStat();
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
                    <MultiSelect v-model="selectedAddressFamily" :options="address_families" optionLabel="name" placeholder="AddressFamily" :maxSelectedLabels="2" class="flex mr-2" />
                    <MultiSelect v-model="selectedTransportProtocol" :options="transport_protocols" optionLabel="name" placeholder="TransportProtocol" :maxSelectedLabels="2" class="mr-2" />
                    <SelectButton class="mr-2" v-model="trafficDisplayType" :options="trafficDisplayTypes" aria-labelledby="traffic-display-type" />
                    <SelectButton class="mr-2" v-model="updateType" :options="updateTypes" optionValue="value" optionLabel="value" dataKey="value" @change="onUpdateTypeChange" aria-labelledby="update-type">
                        <template #option="slotProps">
                            <i :class="slotProps.option.icon"></i>
                            <span>{{slotProps.option.value}}</span>
                        </template>
                    </SelectButton>
                    <Button type="button" icon="pi pi-refresh" outlined :loading="isLoading" @click="GetNetStat" :disabled="autoUpdate" />
                </div>
            </div>
        </template>
        <template #content>
            <DataTable :value="tableData" v-model:selection="selectedHost" :virtualScrollerOptions="{ itemSize: 20 }" selectionMode="single" :dataKey="generateRowKey" @rowSelect="onRowSelect" @rowUnselect="onRowUnselect" size="small" scrollable :scrollHeight="(windowUtil.windowSize.innerHeight-200).toString() + 'px'" tableStyle="min-width: 30rem">
                <Column field="protocol" header="Protocol" sortable></Column>
                <Column field="local_ip_addr" header="Local IP Address" sortable></Column>
                <Column field="local_port" header="Local Port" sortable></Column>
                <Column field="remote_ip_addr" header="Remote IP Address" sortable></Column>
                <Column field="remote_port" header="Remote Port" sortable></Column>
                <div v-if="trafficDisplayType == 'Bandwidth'">
                    <Column field="traffic.formatted_egress_bytes_per_sec" header="Bytes Sent" sortable></Column>
                    <Column field="traffic.formatted_ingress_bytes_per_sec" header="Bytes Recv" sortable></Column>
                </div>
                <div v-else>
                    <Column field="traffic.formatted_sent_bytes" header="Bytes Sent" sortable></Column>
                    <Column field="traffic.formatted_received_bytes" header="Bytes Recv" sortable></Column>
                </div>
                <Column field="process.pid" header="Process ID" sortable></Column>
                <Column field="process.name" header="Process Name" sortable></Column>
            </DataTable>
        </template>
    </Card>
    <Dialog v-model:visible="dialogVisible" :modal="false" :closable="true" header="Detail" :showHeader="true" :breakpoints="{'960px': '75vw', '640px': '100vw'}" :style="{width: '50vw'}">
        <DataTable :value="selectedHostKv" size="small"  scrollable scrollHeight="70vh" tableStyle="min-width: 50rem">
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
