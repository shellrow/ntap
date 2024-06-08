<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { NetRoute } from '../types/network';
import { WindowUtil } from '../util/window';
import { setRoutine } from '../util/routine';
import { DataTableRowSelectEvent } from 'primevue/datatable';
import { SelectButtonChangeEvent } from 'primevue/selectbutton';

const tableData = ref<NetRoute[]>([]);
const isLoading = ref(false);
const selectedHost = ref<any>();
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

const onRowSelect = (_event: DataTableRowSelectEvent) => {

};

const onRowUnselect = (_event: DataTableRowSelectEvent) => {

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
</template>
