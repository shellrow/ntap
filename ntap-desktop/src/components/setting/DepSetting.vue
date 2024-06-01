<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { KVItem } from '../../types/common';

const deps = ref<KVItem[]>([]); 

const checkDependenciesMap = () => {
    invoke<Map<string, boolean>>('get_deps_map').then((res) => {
        for (const [key, value] of Object.entries(res)) {
            let status = value ? 'Installed' : 'Not Installed';
            deps.value.push({key: key, value: status});
        }
    }).catch((err) => {
        console.log(err);
    });
};

onMounted(() => {
    checkDependenciesMap();
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
<Card class="bg-transparent">
    <template #title>Dependencies</template>
    <template #content>
        <div v-for=" dep in deps" :key="dep.key">
            <div class="flex flex-column gap-2 mb-4">
                <InputGroup>
                    <div v-if = "dep.value === 'Installed'">
                        <Button disabled icon="pi pi-check" severity="success" size="small" />
                    </div>
                    <div v-else>
                        <Button disabled icon="pi pi-times" severity="danger" size="small" />
                    </div>
                    <InputText disabled :id="dep.key" v-model="dep.key" size="small" />
                    <Button disabled icon="pi pi-sync" size="small" />
                </InputGroup>
            </div>
        </div>
    </template>
    <template #footer>
        <div class="flex gap-3 mt-1">
        </div>
    </template>
</Card>
</template>
