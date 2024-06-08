<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { KVItem } from '../types/common';
import { NetRoute } from '../types/network';
import { WindowUtil } from '../util/window';
import { setRoutine } from '../util/routine';
import { DataTableRowSelectEvent } from 'primevue/datatable';
import { SelectButtonChangeEvent } from 'primevue/selectbutton';

const tableData = ref<NetRoute[]>([]);
const selectedRouteKv = ref<KVItem[]>([]);
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
            getRoutes(); 
        }
    }
});

const getRoutes = async () => {
    await invoke<NetRoute[]>('get_routes').then((res) => {
        tableData.value = res;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const onRowSelect = (event: DataTableRowSelectEvent) => {
    const r: NetRoute = event.data;
    selectedRouteKv.value = [];
    selectedRouteKv.value.push({key: 'Destination', value: r.destination });
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
    getRoutes();
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
                    <Button type="button" icon="pi pi-refresh" outlined :loading="isLoading" @click="getRoutes" :disabled="autoUpdate" />
                </div>
            </div>
        </template>
        <template #content>
            <DataTable :value="tableData" v-model:selection="selectedHost" :virtualScrollerOptions="{ itemSize: 20 }" selectionMode="single" dataKey="source" @rowSelect="onRowSelect" @rowUnselect="onRowUnselect" size="small" scrollable :scrollHeight="(windowUtil.windowSize.innerHeight-200).toString() + 'px'" tableStyle="min-width: 30rem">
                <Column field="interface_name" header="Interface Name" sortable></Column>
                <Column field="source" header="Source" sortable></Column>
                <Column field="destination" header="Destination" sortable></Column>
                <Column field="netmask" header="Netmask" sortable></Column>
                <Column field="gateway" header="Gateway" sortable></Column>
            </DataTable>
        </template>
    </Card>
    <Dialog v-model:visible="dialogVisible" :modal="false" :closable="true" header="Detail" :showHeader="true" :breakpoints="{'960px': '75vw', '640px': '100vw'}" :style="{width: '45vw'}">
        <DataTable :value="selectedRouteKv"  scrollable scrollHeight="70vh" tableStyle="min-width: 50rem">
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
