<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import DisplaySetting from '../components/setting/DisplaySetting.vue';
import PrivacySetting from '../components/setting/PrivacySetting.vue';
import NetworkSetting from '../components/setting/NetworkSetting.vue';
import PathSetting from './setting/PathSetting.vue';
import LogSetting from './setting/LogSetting.vue';
import DepSetting from './setting/DepSetting.vue';

const innerWidth = ref(window.innerWidth);
const innerHeight = ref(window.innerHeight);

const checkWindowSizeDialog = () => {
    innerWidth.value = window.innerWidth;
    innerHeight.value = window.innerHeight;
};

const emit = defineEmits(["closeSettingDialog"]);

const props = defineProps({
    visible: {
    type: Boolean,
    },
});

const currentComponent = ref("DisplaySetting");

const closeSettingDialog = () => {
    emit("closeSettingDialog");
};

const onVisibleUpdate = () => {
    emit("closeSettingDialog");
};

onMounted(() => {
    window.addEventListener('resize', checkWindowSizeDialog);
});

onUnmounted(() => {
    window.removeEventListener('resize', checkWindowSizeDialog);
});

</script>

<style scoped>

</style>

<template>
<Dialog :visible="props.visible" @update:visible="onVisibleUpdate" modal header="Setting" :style="{ width: '50vw' }" :breakpoints="{ '1199px': '50vw', '575px': '90vw' }">
    <template #header>
        <div class="inline-flex align-items-center justify-content-center gap-2">
            <span class="font-bold white-space-nowrap">Setting</span>
        </div>
    </template>
    <div class="flex relative lg:static surface-ground">
        <div id="setting-sidebar-1" class="" style="width:200px">
            <div class="flex flex-column h-full">
                <div class="overflow-y-auto">
                    <ul class="list-none p-3 m-0">
                        <li>
                            <div v-ripple class="p-3 flex align-items-center justify-content-between text-600 cursor-pointer p-ripple"
                                v-styleclass="{ selector: '@next', enterClass: 'hidden', enterActiveClass: 'slidedown', leaveToClass: 'hidden', leaveActiveClass: 'slideup' }">
                                <span class="font-medium">Preference</span>
                                <i class="pi pi-chevron-down"></i>
                            </div>
                            <ul class="list-none p-0 m-0 overflow-hidden">
                                <li>
                                    <a @click="currentComponent='DisplaySetting'" v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-table mr-2"></i>
                                        <span class="font-medium">Display</span>
                                    </a>
                                </li>
                                <li>
                                    <a @click="currentComponent='PrivacySetting'" v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-eye mr-2"></i>
                                        <span class="font-medium">Privacy</span>
                                    </a>
                                </li>
                                <li>
                                    <a @click="currentComponent='NetworkSetting'" v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-globe mr-2"></i>
                                        <span class="font-medium">Network</span>
                                    </a>
                                </li>
                            </ul>
                        </li>
                    </ul>
                    <ul class="list-none p-3 m-0">
                        <li>
                            <div v-ripple class="p-3 flex align-items-center justify-content-between text-600 cursor-pointer p-ripple"
                                v-styleclass="{ selector: '@next', enterClass: 'hidden', enterActiveClass: 'slidedown', leaveToClass: 'hidden', leaveActiveClass: 'slideup' }">
                                <span class="font-medium">System</span>
                                <i class="pi pi-chevron-down"></i>
                            </div>
                            <ul class="list-none p-0 m-0 overflow-hidden">
                                <li>
                                    <a @click="currentComponent='LogSetting'" v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-list mr-2"></i>
                                        <span class="font-medium">Logging</span>
                                    </a>
                                </li>
                                <li>
                                    <a @click="currentComponent='PathSetting'" v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-folder mr-2"></i>
                                        <span class="font-medium">Path</span>
                                    </a>
                                </li>
                                <li>
                                    <a @click="currentComponent='DepSetting'" v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-cog mr-2"></i>
                                        <span class="font-medium">Dependencies</span>
                                    </a>
                                </li>
                            </ul>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
        <div class="flex flex-column relative flex-auto">
            <div class="p-5 flex flex-column flex-auto">
                <div class="border-2 border surface-border border-round surface-section flex-auto">
                    <ScrollPanel :style="{width: '100%', height: (innerHeight-340).toString() + 'px'}" >
                        <!-- For the performance reason, we use v-if instead of reactively rendering -->
                        <DisplaySetting v-if="currentComponent === 'DisplaySetting'"></DisplaySetting>
                        <PrivacySetting v-if="currentComponent === 'PrivacySetting'"></PrivacySetting>
                        <NetworkSetting v-if="currentComponent === 'NetworkSetting'"></NetworkSetting>
                        <LogSetting v-if="currentComponent === 'LogSetting'"></LogSetting>
                        <PathSetting v-if="currentComponent === 'PathSetting'"></PathSetting>
                        <DepSetting v-if="currentComponent === 'DepSetting'"></DepSetting>
                    </ScrollPanel>
                </div>
            </div>
        </div>
    </div>
    <template #footer>
        <Button type="button" label="Close" @click="closeSettingDialog" outlined></Button>
    </template>
</Dialog>
</template>
