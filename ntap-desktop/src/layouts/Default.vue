<script setup lang="ts">
import {ref, reactive, onMounted, onUnmounted} from 'vue';
import { usePrimeVue } from 'primevue/config';
import { PrimeIcons } from 'primevue/api';
import {useRoute} from 'vue-router';
import {invoke} from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { AppInfo } from '../types/common';
import AboutDialog from '../components/AboutDialog.vue';
import ConfigDialog from '../components/ConfigDialog.vue';
import { DownloadProgress } from '../types/network';
import { AppConfig } from '../types/common';
import { setRoutine } from '../util/routine';

const route = useRoute();
const innerWidth = ref(window.innerWidth);
const innerHeight = ref(window.innerHeight);
const appConfig: AppConfig = reactive(new AppConfig());
const DARK_THEME_NAME = 'lara-dark-teal';
const LIGHT_THEME_NAME = 'lara-light-teal';
const currentTheme = ref(DARK_THEME_NAME);
const currentMode = ref(false);
const currentThemeIcon = ref(PrimeIcons.MOON);
const PrimeVue = usePrimeVue();
const initDialogVisible = ref(false);
const initMessage = ref('Initialising...');
const depsDialogVisible = ref(false);
const depsDialogClosable = ref(false);
const depsProgressMessage = ref('Resolving dependencies...');
const depsStatusMessage = ref('');
const depsErrorMessage = ref('');
const downloadButtonDisabled = ref(false);
const downloadProgress = ref(0);
const downloadContentLength = ref(0);
const aboutDialogVisible = ref(false);
const settingDialogVisible = ref(false);
const visibleRightSidebar = ref(false);

const aboutApp = reactive({
    name: 'Ntap Desktop',
    version: 'v0.1.0',
    release_date: '2024-05-02',
    author: 'shellrow <shellrow@foctet.com>',
    description: 'Cross-platform Network Utilization Statistics tool',
    repository: 'https://github.com/shellrow/ntap',
});

const checkWindowSize = () => {
    innerWidth.value = window.innerWidth;
    innerHeight.value = window.innerHeight;
};

if (!localStorage.theme) {
    localStorage.theme = DARK_THEME_NAME;
}

if (localStorage.theme === DARK_THEME_NAME) {
    currentTheme.value = DARK_THEME_NAME;
    currentThemeIcon.value = PrimeIcons.MOON;
    currentMode.value = false;
} else {
    PrimeVue.changeTheme(DARK_THEME_NAME, LIGHT_THEME_NAME, 'theme-link', () => {});
    currentTheme.value = LIGHT_THEME_NAME;
    currentThemeIcon.value = PrimeIcons.SUN;
    currentMode.value = true;
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

        localStorage.setItem('tick_rate', appConfig.display.tick_rate.toString());
        localStorage.setItem('show_bandwidth', appConfig.display.show_bandwidth.toString());
        localStorage.setItem('hide_private_ip_info', appConfig.privacy.hide_private_ip_info.toString());
        localStorage.setItem('hide_public_ip_info', appConfig.privacy.hide_public_ip_info.toString());
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const systemRoutine = setRoutine({
  interval: 10000,
  callback: () => { 
        getAppConfig();
    }
});

const changeMode = () => {
    let prevTheme = LIGHT_THEME_NAME;
    let nextTheme = DARK_THEME_NAME;
    if (currentMode.value) {
        prevTheme = LIGHT_THEME_NAME;
        nextTheme = DARK_THEME_NAME;
        currentTheme.value = DARK_THEME_NAME; 
        currentThemeIcon.value = PrimeIcons.MOON;
    }else{
        prevTheme = DARK_THEME_NAME;
        nextTheme = LIGHT_THEME_NAME;
        currentTheme.value = LIGHT_THEME_NAME;
        currentThemeIcon.value = PrimeIcons.SUN;
    }
    PrimeVue.changeTheme(prevTheme, nextTheme, 'theme-link', () => {});
    localStorage.theme = currentTheme.value;
    currentMode.value = !currentMode.value;
};

const startBackgroundTask = async () => {
    const unlisten = await listen<string>('init', (event) => {
        initMessage.value = event.payload;
    });
    initDialogVisible.value = true;
    invoke('start_background_task').then((_res) => {
        initDialogVisible.value = false;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        unlisten();
    });
};

const checkDependenciesMap = () => {
    invoke<Map<string, boolean>>('get_deps_map').then((res) => {
        const deps :Map<string, boolean> = res;
        if (deps.size > 0 && !deps.get('npcap')) {
            depsDialogVisible.value = true;
        }
    }).catch((err) => {
        console.log(err);
    });
};

// Check dependencies
// true: resolved (or no-deps), false: not resolved
const checkDependencies = () => {
    let deps = new Map<string, boolean>();
    invoke<Map<string, boolean>>('get_deps_map').then((res) => {
        deps = res;
    }).catch((err) => {
        console.log(err);
    });
    for (const [_key, value] of deps) {
        if (!value) {
            return false;
        }
    }
    return true;
};


const resolveDep = async () => {
    // Download installer
    const unlisten = await listen<DownloadProgress>('download_progress', (event) => {
        const data: DownloadProgress = event.payload;
        if ('ContentLength' in data) {
            downloadContentLength.value = data.ContentLength;
            depsProgressMessage.value = `Downloading npcap installer... (${data.ContentLength} bytes)`;
        } else if ('Downloaded' in data) {
            if (downloadContentLength.value > 0) {
                downloadProgress.value = (data.Downloaded / downloadContentLength.value) *100;
            }else{
                downloadProgress.value = 0;
            }
        }
    });
    depsProgressMessage.value = 'Downloading npcap installer...';
    await invoke('download_dep', {softwareName: "npcap"}).then((res) => {
        downloadProgress.value = 100;
        downloadButtonDisabled.value = true;
        depsProgressMessage.value = `Download complete (${res} bytes)`;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        unlisten();
    });
    // Run installer
    if (downloadProgress.value === 100) {
        await invoke('run_dep_installer', {softwareName: "npcap"}).then((_res) => {
            
        }).catch((err) => {
            console.log(err);
        });
    }
    if (checkDependencies()) {
        depsDialogClosable.value = true;
        depsStatusMessage.value = 'Great! Dependencies resolved. You can close this dialog.';
    }else{
        depsDialogClosable.value = false;
        depsErrorMessage.value = 'Failed to resolve dependencies. Please try again or manually download and install npcap from https://npcap.com';
    }
};

const getAppInfo = () => {
    invoke<AppInfo>('get_app_info').then((res) => {
        //aboutApp.name = res.name;
        aboutApp.description = res.description;
        aboutApp.version = `v${res.version}`;
        aboutApp.release_date = res.release_date;
        aboutApp.repository = res.repository;
    }).catch((err) => {
        console.log(err);
    }).finally(() => {
        
    });
}

const initApp = async () => {
    await checkDependenciesMap();
    // wait for dependencies to be resolved
    while (depsDialogVisible.value) {
        await new Promise((resolve) => setTimeout(resolve, 1000));
    }
    await startBackgroundTask();
};

const openAboutDialog = () => {
    aboutDialogVisible.value = true;
};

const closeAboutDialog = () => {
    aboutDialogVisible.value = false;
};

const openSettingDialog = () => {
    settingDialogVisible.value = true;
};

const closeSettingDialog = () => {
    settingDialogVisible.value = false;
};

onMounted(() => {
    window.addEventListener('resize', checkWindowSize);
    getAppInfo();
    initApp();
    systemRoutine.start();
});

onUnmounted(() => {
    window.removeEventListener('resize', checkWindowSize);
    systemRoutine.stop();
});
</script>

<style scoped>
/* color palette from <https://github.com/vuejs/theme> */
:root {
    --vt-c-white: #ffffff;
    --vt-c-white-soft: #f8f8f8;
    --vt-c-white-mute: #f2f2f2;

    --vt-c-black: #181818;
    --vt-c-black-soft: #222222;
    --vt-c-black-mute: #282828;

    --vt-c-indigo: #2c3e50;

    --vt-c-divider-light-1: rgba(60, 60, 60, 0.29);
    --vt-c-divider-light-2: rgba(60, 60, 60, 0.12);
    --vt-c-divider-dark-1: rgba(84, 84, 84, 0.65);
    --vt-c-divider-dark-2: rgba(84, 84, 84, 0.48);

    --vt-c-text-light-1: var(--vt-c-indigo);
    --vt-c-text-light-2: rgba(60, 60, 60, 0.66);
    --vt-c-text-dark-1: var(--vt-c-white);
    --vt-c-text-dark-2: rgba(235, 235, 235, 0.64);
}

/* semantic color variables for this project */
:root {
    --color-background: var(--vt-c-white);
    --color-background-soft: var(--vt-c-white-soft);
    --color-background-mute: var(--vt-c-white-mute);

    --color-border: var(--vt-c-divider-light-2);
    --color-border-hover: var(--vt-c-divider-light-1);

    --color-heading: var(--vt-c-text-light-1);
    --color-text: var(--vt-c-text-light-1);

    --section-gap: 160px;
}

@media (prefers-color-scheme: dark) {
    :root {
        --color-background: var(--vt-c-black);
        --color-background-soft: var(--vt-c-black-soft);
        --color-background-mute: var(--vt-c-black-mute);

        --color-border: var(--vt-c-divider-dark-2);
        --color-border-hover: var(--vt-c-divider-dark-1);

        --color-heading: var(--vt-c-text-dark-1);
        --color-text: var(--vt-c-text-dark-2);
    }
}

*,
*::before,
*::after {
    box-sizing: border-box;
    margin: 0;
    position: relative;
    font-weight: normal;
}

body {
    min-height: 100vh;
    color: var(--color-text);
    background: var(--color-background);
    transition: color 0.5s, background-color 0.5s;
    line-height: 1.6;
    font-family: Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu,
    Cantarell, 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
    font-size: 12px;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
}

a,
.green {
  text-decoration: none;
  color: hsla(160, 100%, 37%, 1);
  transition: 0.4s;
}

@media (hover: hover) {
  a:hover {
    background-color: hsla(160, 100%, 37%, 0.2);
  }
}

</style>

<template>
<div class="min-h-screen flex relative lg:static surface-ground">
    <div id="app-sidebar-1" class="surface-section h-screen hidden lg:block flex-shrink-0 absolute lg:static left-0 top-0 z-1 border-right-1 surface-border select-none" style="width:200px; font-size: 14px;">
        <div class="flex flex-column h-full">
            <div class="flex align-items-center px-5 flex-shrink-0" style="height:60px">
                <a v-ripple class="cursor-pointer block lg:hidden text-700 mr-3 mt-1 p-ripple"
                    v-styleclass="{ selector: '#app-sidebar-1', enterClass: 'hidden', enterActiveClass: 'fadeinleft', leaveToClass: 'hidden', leaveActiveClass: 'fadeoutleft', hideOnOutsideClick: true }">
                    <i class="pi pi-bars text-4xl"></i>
                </a>
                <span class="font-medium" style="color: var(--highlight-text-color)">
                    Ntap
                </span>
            </div>
            <div class="overflow-y-auto">
                <ul class="list-none p-3 m-0">
                    <li>
                        <router-link to="/">
                            <a v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                <i class="pi pi-home mr-2"></i>
                                <span class="font-medium">Overview</span>
                            </a>
                        </router-link>
                    </li>
                    <li>
                        <a v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple"
                            v-styleclass="{ selector: '@next', enterClass: 'hidden', enterActiveClass: 'slidedown', leaveToClass: 'hidden', leaveActiveClass: 'slideup' }">
                            <i class="pi pi-desktop mr-2"></i>
                            <span class="font-medium">Monitoring</span>
                            <i class="pi pi-chevron-down ml-auto"></i>
                        </a>
                        <ul class="list-none py-0 pl-3 pr-0 m-0 overflow-y-hidden transition-all transition-duration-400 transition-ease-in-out">
                            <li>
                                <router-link to="/remote">
                                    <a v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-globe mr-2"></i>
                                        <span class="font-medium">RemoteAddress</span>
                                    </a>
                                </router-link>
                            </li>
                            <li>
                                <router-link to="/socket">
                                    <a v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-link mr-2"></i>
                                        <span class="font-medium">Socket</span>
                                    </a>
                                </router-link>
                            </li>
                            <li>
                                <router-link to="/process">
                                    <a v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-list mr-2"></i>
                                        <span class="font-medium">Process</span>
                                    </a>
                                </router-link>
                            </li>
                        </ul>
                    </li>
                    <li>
                        <a v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple"
                            v-styleclass="{ selector: '@next', enterClass: 'hidden', enterActiveClass: 'slidedown', leaveToClass: 'hidden', leaveActiveClass: 'slideup' }">
                            <i class="pi pi-microchip mr-2"></i>
                            <span class="font-medium">System</span>
                            <i class="pi pi-chevron-down ml-auto"></i>
                        </a>
                        <ul class="list-none py-0 pl-3 pr-0 m-0 hidden overflow-y-hidden transition-all transition-duration-400 transition-ease-in-out">
                            <li>
                                <router-link to="/interface">
                                    <a v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-arrows-h mr-2"></i>
                                        <span class="font-medium">Interface</span>
                                    </a>
                                </router-link>
                            </li>
                            <li>
                                <router-link to="/">
                                    <a v-ripple class="flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                                        <i class="pi pi-directions mr-2"></i>
                                        <span class="font-medium">Route</span>
                                    </a>
                                </router-link>
                            </li>
                        </ul>
                    </li>
                </ul>
            </div>
            <div class="mt-auto">
                <hr class="mb-3 mx-3 border-top-1 border-none surface-border" />
                <a @click="openSettingDialog" v-ripple class="m-3 flex align-items-center cursor-pointer p-3 border-round text-700 hover:surface-100 transition-duration-150 transition-colors p-ripple">
                    <i class="pi pi-cog mr-2"></i>
                    <span class="font-medium">Settings</span>
                </a>
            </div>
        </div>
    </div>
    <div class="min-h-screen flex flex-column relative flex-auto" style="z-index: 0;">
        <div class="flex justify-content-between align-items-center px-5 surface-0 border-bottom-1 surface-border relative lg:static" style="height:60px">
            <div class="flex">
                <a v-ripple class="cursor-pointer block lg:hidden text-700 mr-3 mt-1 p-ripple"
                    v-styleclass="{ selector: '#app-sidebar-1', enterClass: 'hidden', enterActiveClass: 'fadeinleft', leaveToClass: 'hidden', leaveActiveClass: 'fadeoutleft', hideOnOutsideClick: true }">
                    <i class="pi pi-bars text-4xl"></i>
                </a>
                <span class="font-medium mt-1" style="color: var(--highlight-text-color);font-size: 14px;">
                    {{ route.name }}
                </span>
            </div>
            <div class="flex">
                <Button label="" :icon="currentThemeIcon" @click="changeMode" severity="secondary" outlined class="text-base mr-2" />
                <Button label="" icon="pi pi-bell" severity="secondary" outlined class="text-base mr-2" @click="visibleRightSidebar = true" />
                <Button @click="openAboutDialog" label="" icon="pi pi-question-circle" severity="secondary" outlined class="text-base mr-2" />
            </div>
        </div>
        <div class="p-2 flex flex-column flex-auto" style="z-index: -1;">
            <!--<div class="border-2 border-dashed surface-border border-round surface-section flex-auto">-->
            <div class="surface-section flex-auto">
                <ScrollPanel :style="{width: '100%', height: (innerHeight-100).toString() + 'px'}" >
                    <router-view></router-view>
                </ScrollPanel>
            </div>
        </div>
    </div>
    <Sidebar v-model:visible="visibleRightSidebar" header="Notification" position="right">
        <p>
            No notifications
        </p>
    </Sidebar>
</div>
<AboutDialog
    :visible="aboutDialogVisible"
    @close-about-dialog="closeAboutDialog"
/>
<ConfigDialog
    :visible="settingDialogVisible"
    @close-setting-dialog="closeSettingDialog"
/>

<Dialog v-model:visible="initDialogVisible" :closable="false" modal header="Initializing..." :style="{ width: '60vw' }" :breakpoints="{ '1199px': '60vw', '575px': '90vw' }">
    <p class="m-0">
        {{ initMessage }}
    </p>
</Dialog>

<Dialog v-model:visible="depsDialogVisible" :closable="depsDialogClosable" modal header="Resolve Dependencies" :style="{ width: '60vw' }" :breakpoints="{ '1199px': '60vw', '575px': '90vw' }">
    <p class="m-0 font-bold block mb-3 text-lg">
        On windows, npcap is required to capture network packets. Please install npcap.
    </p>
    <p class="m-0 mb-3">
        Click the button below to download npcap installer from <a href="https://npcap.com" target="_blank">https://npcap.com</a> and run the installer.
    </p>
    <Button class="mb-3" label="Install npcap" icon="pi pi-download" @click="resolveDep" :disabled="downloadButtonDisabled" />
    <div class="flex-auto mb-3">
        <label for="deps-status" class="font-bold block"> {{ depsProgressMessage }} </label>
        <ProgressBar :value="downloadProgress"></ProgressBar>
    </div>
    <div class="flex-auto">
        <label for="deps-error" class="font-bold block text-500 text-xl"> {{ depsStatusMessage }} </label>
    </div>
    <div class="flex-auto">
        <label for="deps-error" class="font-bold block text-red-500 text-xl"> {{ depsErrorMessage }} </label>
    </div>
</Dialog>

</template>
