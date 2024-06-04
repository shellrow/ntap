<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { KVItem } from '../types/common';
import { NetworkInterface } from '../types/network';
import { WindowUtil } from '../util/window';
import { setRoutine } from '../util/routine';
import { DataTableRowSelectEvent } from 'primevue/datatable';
import { SelectButtonChangeEvent } from 'primevue/selectbutton';

const tableData = ref<NetworkInterface[]>([]);
const selectedInterfaceKv = ref<KVItem[]>([]);
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

const routine = setRoutine({
  interval: 1000,
  callback: () => { 
        if (autoUpdate.value) {
            getInterfaces(); 
        }
    }
});

const getInterfaces = async () => {
    await invoke<NetworkInterface[]>('get_interfaces').then((res) => {
        tableData.value = res;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const onRowSelect = (event: DataTableRowSelectEvent) => {
    const iface: NetworkInterface = event.data;
    selectedInterfaceKv.value = [];
    selectedInterfaceKv.value.push({key: 'Index', value: iface.index.toString() });
    selectedInterfaceKv.value.push({key: 'Name', value: iface.name });
    selectedInterfaceKv.value.push({key: 'Friendly Name', value: iface.friendly_name || ''});
    selectedInterfaceKv.value.push({key: 'Type', value: iface.if_type });
    selectedInterfaceKv.value.push({key: 'MAC', value: iface.mac_addr || '' });
    selectedInterfaceKv.value.push({key: 'IPv4', value: iface.ipv4.map((item) => item.addr).join(', ') });
    selectedInterfaceKv.value.push({key: 'IPv6', value: iface.ipv6.map((item) => item.addr).join(', ') });
    selectedInterfaceKv.value.push({key: 'Gateway MAC', value: iface.gateway?.mac_addr || '' });
    selectedInterfaceKv.value.push({key: 'Gateway IPv4', value: iface.gateway?.ipv4.join(', ') || '' });
    selectedInterfaceKv.value.push({key: 'Gateway IPv6', value: iface.gateway?.ipv6.join(', ') || '' });
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
    getInterfaces();
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
                    <SelectButton class="mr-2" v-model="updateType" :options="updateTypes" optionValue="value" optionLabel="value" dataKey="value" @change="onUpdateTypeChange" aria-labelledby="update-type">
                        <template #option="slotProps">
                            <i :class="slotProps.option.icon"></i>
                            <span>{{slotProps.option.value}}</span>
                        </template>
                    </SelectButton>
                    <Button type="button" icon="pi pi-refresh" outlined :loading="isLoading" @click="getInterfaces" :disabled="autoUpdate" />
                </div>
            </div>
        </template>
        <template #content>
            <DataTable :value="tableData" v-model:selection="selectedHost" :virtualScrollerOptions="{ itemSize: 20 }" selectionMode="single" dataKey="index" @rowSelect="onRowSelect" @rowUnselect="onRowUnselect" size="small" scrollable :scrollHeight="(windowUtil.windowSize.innerHeight-200).toString() + 'px'" tableStyle="min-width: 30rem">
                <Column field="index" header="Index" sortable></Column>
                <Column field="name" header="Name" sortable></Column>
                <Column field="friendly_name" header="Friendly Name" sortable></Column>
                <Column field="if_type" header="Type" sortable></Column>
                <Column field="mac_addr" header="MAC Address" sortable></Column>
                <Column header="IPv4">
                    <template #body="slotProps">
                        <div v-if="slotProps.data.ipv4.length > 0">
                            {{slotProps.data.ipv4[0].addr}}
                        </div>
                    </template>
                </Column>
                <Column header="IPv6">
                    <template #body="slotProps">
                        <div v-if="slotProps.data.ipv6.length > 0">
                            {{slotProps.data.ipv6[0].addr}}
                        </div>
                    </template>
                </Column>
            </DataTable>
        </template>
    </Card>
    <Dialog v-model:visible="dialogVisible" :modal="false" :closable="true" header="Detail" :showHeader="true" :breakpoints="{'960px': '75vw', '640px': '100vw'}" :style="{width: '45vw'}">
        <DataTable :value="selectedInterfaceKv"  scrollable scrollHeight="70vh" tableStyle="min-width: 50rem">
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
