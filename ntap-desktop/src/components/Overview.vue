<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { PrimeIcons } from 'primevue/api';
import { setRoutine } from '../util/routine';
import { Overview, ServiceDisplayInfo, IpInfo, NetworkInterface, IpInfoDisplayData } from '../types/network';
import { KVItem } from '../types/common';
import { useToast } from "primevue/usetoast";
import { SelectButtonChangeEvent } from 'primevue/selectbutton';

const toast = useToast();
const autoUpdate = ref(true);
const updateType = ref('Auto');
const updateTypes = ref(
    [
        {value: 'Auto', icon: 'pi pi-play'},
        {value: 'Manual', icon: 'pi pi-pause'}
    ]
);
const overview = ref<Overview>();
const ipinfo = ref<IpInfo>();
const ipv4info = ref<IpInfo>();
const networkInterface = ref<NetworkInterface>();
const isLoading = ref(false);
const tickRate = ref(1000);
const showPrivateIpv4 = ref(false);
const privateIpv4VisibleIcon = ref(PrimeIcons.EYE_SLASH);
const showPrivateIpv6 = ref(false);
const privateIpv6VisibleIcon = ref(PrimeIcons.EYE_SLASH);
const showPublicIpv4 = ref(false);
const publicIpv4VisibleIcon = ref(PrimeIcons.EYE_SLASH);
const showPublicIpv6 = ref(false);
const publicIpv6VisibleIcon = ref(PrimeIcons.EYE_SLASH);
const trafficDisplayType = ref('Total');
const trafficDisplayTypes = ref(['Total', 'Bandwidth']);
const interfaceDialogVisible = ref(false);
const interfaceKv = ref<KVItem[]>([]);
const ipInfoDialogVisible = ref(false);
const ipInfoKv = ref<KVItem[]>([]);
const ipInfoDisplayData = reactive<IpInfoDisplayData>(
    {
        ipv4: 'N/A',
        ipv4_network: 'N/A',
        ipv6: 'N/A',
        ipv6_network: 'N/A',
        host_name: 'N/A',
        asn: 'N/A',
        as_name: 'N/A',
        country_code: 'N/A',
        country_name: 'N/A'
    }
);

const routine = setRoutine({
  interval: tickRate.value,
  callback: () => { 
        if (autoUpdate.value) {
            GetOverview(); 
        }
    }
});

const systemRoutine = setRoutine({
  interval: 1000,
  callback: () => { 
        getAppConfig();
    }
});

const getAppConfig = async () => {
    tickRate.value = localStorage.getItem('tick_rate') ? parseInt(localStorage.getItem('tickRate')!) : 1000;
}

const setDefaultOptions = () => {
    showPrivateIpv4.value = localStorage.getItem('hide_private_ip_info') ? localStorage.getItem('hide_private_ip_info') === 'false' : false;
    showPrivateIpv6.value = localStorage.getItem('hide_private_ip_info') ? localStorage.getItem('hide_private_ip_info') === 'false' : false;
    showPublicIpv4.value = localStorage.getItem('hide_public_ip_info') ? localStorage.getItem('hide_public_ip_info') === 'false' : false;
    showPublicIpv6.value = localStorage.getItem('hide_public_ip_info') ? localStorage.getItem('hide_public_ip_info') === 'false' : false;
    privateIpv4VisibleIcon.value = showPrivateIpv4.value ? PrimeIcons.EYE : PrimeIcons.EYE_SLASH;
    privateIpv6VisibleIcon.value = showPrivateIpv6.value ? PrimeIcons.EYE : PrimeIcons.EYE_SLASH;
    publicIpv4VisibleIcon.value = showPublicIpv4.value ? PrimeIcons.EYE : PrimeIcons.EYE_SLASH;
    publicIpv6VisibleIcon.value = showPublicIpv6.value ? PrimeIcons.EYE : PrimeIcons.EYE_SLASH;
    if (localStorage.getItem('show_bandwidth') === 'true') {
        trafficDisplayType.value = 'Bandwidth';
    }else{
        trafficDisplayType.value = 'Total';
    }
}

const GetOverview = async() => {
    const result = await invoke<Overview>('get_overview');
    overview.value = result;
}

const GetIpInfo = async() => {
    invoke<IpInfo>('get_self_ip_info').then((result) => {
        ipinfo.value = result;
        if (result.ip_version === 'v4') {
            ipInfoDisplayData.ipv4 = result.ip_addr;
            ipInfoDisplayData.ipv4_network = result.network;
        } else {
            ipInfoDisplayData.ipv6 = result.ip_addr;
            ipInfoDisplayData.ipv6_network = result.network;
        }
        if (result.host_name !== '') {
            ipInfoDisplayData.host_name = result.host_name;
        }
        ipInfoDisplayData.asn = result.asn;
        ipInfoDisplayData.as_name = result.as_name;
        ipInfoDisplayData.country_code = result.country_code;
        ipInfoDisplayData.country_name = result.country_name;

    }).catch((error) => {
        console.error(error);
    });
}

const GetIpv4Info = async() => {
    invoke<IpInfo>('get_self_ipv4_info').then((result) => {
        ipv4info.value = result;
        ipInfoDisplayData.ipv4 = result.ip_addr;
        ipInfoDisplayData.ipv4_network = result.network;
        if (result.host_name !== '') {
            ipInfoDisplayData.host_name = result.host_name;
        }
        if (result.asn !== '') {
            ipInfoDisplayData.asn = result.asn;
            ipInfoDisplayData.as_name = result.as_name;
        }
        if (result.country_code !== '') {
            ipInfoDisplayData.country_code = result.country_code;
            ipInfoDisplayData.country_name = result.country_name;
        }
    }).catch((error) => {
        console.error(error);
    });
}

const GetNetworkInterface = async() => {
    invoke<NetworkInterface>('get_default_interface').then((result) => {
        networkInterface.value = result;
        interfaceKv.value = [];
        interfaceKv.value.push({key: 'Index', value: result.index.toString()});
        interfaceKv.value.push({key: 'Name', value: result.name});
        interfaceKv.value.push({key: 'Friendly Name', value: result.friendly_name ?? 'N/A'});
        interfaceKv.value.push({key: 'Description', value: result.description ?? 'N/A'});
        interfaceKv.value.push({key: 'Type', value: result.if_type});
        interfaceKv.value.push({key: 'MAC Address', value: result.mac_addr ?? 'N/A'});
        interfaceKv.value.push({key: 'IPv4', value: result.ipv4.map((item) => item.addr).join(', ')});
        interfaceKv.value.push({key: 'IPv6', value: result.ipv6.map((item) => item.addr).join(', ')});
        interfaceKv.value.push({key: 'Transmit Speed', value: result.transmit_speed ? result.transmit_speed.toString() : 'N/A'});
        interfaceKv.value.push({key: 'Receive Speed', value: result.receive_speed ? result.receive_speed.toString() : 'N/A'});
        interfaceKv.value.push({key: 'Gateway MAC Address', value: result.gateway?.mac_addr ?? 'N/A'});
        interfaceKv.value.push({key: 'Gateway IPv4', value: result.gateway?.ipv4.join(', ') ?? 'N/A'});
        interfaceKv.value.push({key: 'Gateway IPv6', value: result.gateway?.ipv6.join(', ') ?? 'N/A'});

        interfaceKv.value.push({key: 'DNS Servers', value: result.dns_servers.join(', ')});
    }).catch((error) => {
        console.error(error);
    });
}

const generateProtocolPortKey = (service: ServiceDisplayInfo) => {
    return service.port + '/' + service.protocol;
}

const togglePrivateIpv4Visible = () => {
    showPrivateIpv4.value = !showPrivateIpv4.value;
    privateIpv4VisibleIcon.value = showPrivateIpv4.value ? PrimeIcons.EYE : PrimeIcons.EYE_SLASH;
}

const togglePrivateIpv6Visible = () => {
    showPrivateIpv6.value = !showPrivateIpv6.value;
    privateIpv6VisibleIcon.value = showPrivateIpv6.value ? PrimeIcons.EYE : PrimeIcons.EYE_SLASH;
}

const togglePublicIpv4Visible = () => {
    showPublicIpv4.value = !showPublicIpv4.value;
    publicIpv4VisibleIcon.value = showPublicIpv4.value ? PrimeIcons.EYE : PrimeIcons.EYE_SLASH;
}

const togglePublicIpv6Visible = () => {
    showPublicIpv6.value = !showPublicIpv6.value;
    publicIpv6VisibleIcon.value = showPublicIpv6.value ? PrimeIcons.EYE : PrimeIcons.EYE_SLASH;
}

const onIpInfoDetailClick = () => {
    ipInfoDialogVisible.value = true;
    ipInfoKv.value = [];
    ipInfoKv.value.push({key: 'IPv4', value: ipInfoDisplayData.ipv4});
    ipInfoKv.value.push({key: 'IPv4 Network', value: ipInfoDisplayData.ipv4_network});
    ipInfoKv.value.push({key: 'IPv6', value: ipInfoDisplayData.ipv6});
    ipInfoKv.value.push({key: 'IPv6 Network', value: ipInfoDisplayData.ipv6_network});
    //ipInfoKv.value.push({key: 'Host Name', value: ipInfoDisplayData.host_name});
    ipInfoKv.value.push({key: 'ASN', value: ipInfoDisplayData.asn});
    ipInfoKv.value.push({key: 'AS Name', value: ipInfoDisplayData.as_name});
    ipInfoKv.value.push({key: 'Country Code', value: ipInfoDisplayData.country_code});
    ipInfoKv.value.push({key: 'Country Name', value: ipInfoDisplayData.country_name});
}

const clickCopyPrivateIpv4 = () => {
    navigator.clipboard.writeText(networkInterface.value?.ipv4[0].addr ?? '').then(() => {
        toast.add({ severity: 'success', summary: 'Success', detail: 'Private IPv4 Address copied!', life: 2000 });
    })
    .catch(e => {
        console.log(e);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to copy Private IPv4 Address!', life: 2000 });
    });
}

const clickCopyPrivateIpv6 = () => {
    navigator.clipboard.writeText(networkInterface.value?.ipv6[0].addr ?? '').then(() => {
        toast.add({ severity: 'success', summary: 'Success', detail: 'Private IPv6 Address copied!', life: 2000 });
    })
    .catch(e => {
        console.log(e);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to copy Private IPv6 Address!', life: 2000 });
    });
}

const clickCopyPublicIpv4 = () => {
    navigator.clipboard.writeText(ipv4info.value?.ip_addr ?? '').then(() => {
        toast.add({ severity: 'success', summary: 'Success', detail: 'Public IPv4 Address copied!', life: 2000 });
    })
    .catch(e => {
        console.log(e);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to copy Public IPv4 Address!', life: 2000 });
    });
}

const clickCopyPublicIpv6 = () => {
    navigator.clipboard.writeText(ipinfo.value?.ip_addr ?? '').then(() => {
        toast.add({ severity: 'success', summary: 'Success', detail: 'Public IPv6 Address copied!', life: 2000 });
    })
    .catch(e => {
        console.log(e);
        toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to copy Public IPv6 Address!', life: 2000 });
    });
}

const onUpdateTypeChange = (event: SelectButtonChangeEvent) => {
    if (event.value === 'Auto') {
        autoUpdate.value = true;
    } else {
        autoUpdate.value = false;
    }
}

const UpdateAll = () => {
    GetNetworkInterface();
    GetOverview();
    GetIpInfo();
    GetIpv4Info();
}

onMounted(() => {
    getAppConfig();
    setDefaultOptions();
    UpdateAll();
    routine.start();
    systemRoutine.start();
});

onUnmounted(() => {
    routine.stop();
    systemRoutine.stop();
});

</script>

<style scoped>
.border-none {
    border: unset;
}
.background-none {
    background-color: transparent;
}
.small-button {
    height: 2.0rem
}
</style>

<template>
    <Toast />
    <div class="flex flex-column flex-auto">
        <div class="p-5">
            <div class="flex justify-content-between mb-4">
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
                    <Button type="button" icon="pi pi-refresh" outlined :loading="isLoading" @click="UpdateAll" :disabled="autoUpdate" />
                </div>
            </div>
            <div class="grid">
                <!--<div class="col-12 lg:col-6 xl:col-3">-->
                <div class="col-12 lg:col-6">
                    <div class="surface-card shadow-2 p-3 border-1 border-50 border-round">
                        <div class="flex justify-content-between mb-3">
                            <div>
                                <span class="block text-500 font-medium mb-3 text-lg">Default Interface: [{{ networkInterface?.index }}] {{ networkInterface?.name }}</span>
                                <div class="text-900 font-medium text-xl">
                                    <InputGroup>
                                        <Button :icon="privateIpv4VisibleIcon" severity="info" text @click="togglePrivateIpv4Visible" />
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">IPv4</InputGroupAddon>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">
                                            {{ showPrivateIpv4 ? networkInterface?.ipv4[0].addr : 'xxx.xxx.xxx.xxx' }}
                                        </InputGroupAddon>
                                        <Button icon="pi pi-copy" severity="info" text @click="clickCopyPrivateIpv4" />
                                    </InputGroup>
                                </div>
                                <div class="text-900 font-medium text-xl">
                                    <InputGroup>
                                        <Button :icon="privateIpv6VisibleIcon" severity="info" text @click="togglePrivateIpv6Visible" />
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">IPv6</InputGroupAddon>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">
                                            {{ showPrivateIpv6 ? networkInterface?.ipv6[0].addr : 'xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx' }}
                                        </InputGroupAddon>
                                        <Button icon="pi pi-copy" severity="info" text @click="clickCopyPrivateIpv6" />
                                    </InputGroup>
                                </div>
                            </div>
                            <div class="flex align-items-center justify-content-center bg-orange-100 border-round" style="width:2.5rem;height:2.5rem">
                                <i class="pi pi-cog text-orange-500 text-xl"></i>
                            </div>
                        </div>
                        <div class="flex justify-content-between">
                            <span class="text-500">Click eye icon to toggle visibility</span>
                            <Button label="Detail" severity="primary" @click="interfaceDialogVisible = true" />
                        </div>
                        
                        <!-- <span class="text-500">Index: </span>
                        <span class="text-green-500 font-medium">{{ networkInterface?.index }}</span> -->
                    </div>
                </div>
                <div class="col-12 lg:col-6">
                    <div class="surface-card shadow-2 p-3 border-1 border-50 border-round">
                        <div class="flex justify-content-between mb-3">
                            <div>
                                <span class="block text-500 font-medium mb-3 text-lg">Public IP Address</span>
                                <div class="text-900 font-medium text-xl">
                                    <InputGroup>
                                        <Button :icon="publicIpv4VisibleIcon" severity="info" text @click="togglePublicIpv4Visible" />
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">IP{{ ipv4info?.ip_version }}</InputGroupAddon>
                                        <!-- If showPublicIpv4 is false, mask IP Address  -->
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">
                                            {{ showPublicIpv4 ? ipv4info?.ip_addr : 'xxx.xxx.xxx.xxx' }}
                                        </InputGroupAddon>
                                        <Button icon="pi pi-copy" severity="info" text @click="clickCopyPublicIpv4" />
                                    </InputGroup>
                                </div>
                                <div class="text-900 font-medium text-xl">
                                    <InputGroup>
                                        <Button :icon="publicIpv6VisibleIcon" severity="info" text @click="togglePublicIpv6Visible" />
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">IP{{ ipinfo?.ip_version }}</InputGroupAddon>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">
                                            {{ showPublicIpv6 ? ipinfo?.ip_addr : 'xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx' }}
                                        </InputGroupAddon>
                                        <Button icon="pi pi-copy" severity="info" text @click="clickCopyPublicIpv6" />
                                    </InputGroup>
                                </div>
                            <!-- <div class="text-900 font-medium text-xl">IP{{ ipinfo?.ip_version }}: {{ ipinfo?.ip_addr }}</div>
                            <div class="text-900 font-medium text-xl">IP{{ ipv4info?.ip_version }}: {{ ipv4info?.ip_addr }}</div> -->
                            </div>
                            <div class="flex align-items-center justify-content-center bg-blue-100 border-round" style="width:2.5rem;height:2.5rem">
                                <i class="pi pi-globe text-blue-500 text-xl"></i>
                            </div>
                        </div>
                        <div class="flex justify-content-between">
                            <span class="text-500">Click eye icon to toggle visibility</span>
                            <Button label="Detail" severity="primary" @click="onIpInfoDetailClick" />
                        </div>
                        <!-- <span class="text-green-500 font-medium"> {{ ipinfo?.country_code + " " }} </span>
                        <span class="text-500">{{ ipinfo?.as_name }}</span> -->
                    </div>
                </div>

                <div class="col-12 lg:col-6">
                    <div class="surface-card shadow-2 p-3 border-1 border-50 border-round">
                        <div class="flex justify-content-between mb-3">
                            <div>
                                <span class="block text-500 font-medium mb-3 text-lg">Ingress</span>
                                <div class="text-900 font-medium text-xl">
                                    <InputGroup>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">Packets</InputGroupAddon>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">{{ trafficDisplayType == "Bandwidth" ? overview?.traffic.formatted_ingress_packets_per_sec : overview?.traffic.packet_received }}</InputGroupAddon>
                                    </InputGroup>
                                </div>
                                <div class="text-900 font-medium text-xl">
                                    <InputGroup>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">Bytes</InputGroupAddon>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">{{ trafficDisplayType == "Bandwidth" ? overview?.traffic.formatted_ingress_bytes_per_sec : overview?.traffic.formatted_received_bytes }}</InputGroupAddon>
                                    </InputGroup>
                                </div>
                                <!-- <div class="text-900 font-medium text-xl">{{ overview?.traffic.bytes_received }} bytes</div> -->
                            </div>
                            <div class="flex align-items-center justify-content-center bg-cyan-100 border-round" style="width:2.5rem;height:2.5rem">
                                <i class="pi pi-download text-cyan-500 text-xl"></i>
                            </div>
                        </div>
                        <div class="flex justify-content-between">
                            <span class="text-500">Download traffic summary</span>
                            <span class="text-green-500 font-medium">{{ trafficDisplayType }}</span>
                        </div>
                        
                        <!-- <span class="text-green-500 font-medium">{{ overview?.traffic.packet_received }}</span>
                        <span class="text-500"> packets received</span> -->
                    </div>
                </div>
                <div class="col-12 lg:col-6">
                    <div class="surface-card shadow-2 p-3 border-1 border-50 border-round">
                        <div class="flex justify-content-between mb-3">
                            <div>
                                <span class="block text-500 font-medium mb-3 text-lg">Egress</span>
                                <div class="text-900 font-medium text-xl">
                                    <InputGroup>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">Packets</InputGroupAddon>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">{{ trafficDisplayType == "Bandwidth" ? overview?.traffic.formatted_egress_packets_per_sec : overview?.traffic.packet_sent }}</InputGroupAddon>
                                    </InputGroup>
                                </div>
                                <div class="text-900 font-medium text-xl">
                                    <InputGroup>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">Bytes</InputGroupAddon>
                                        <InputGroupAddon class="text-900 font-medium border-none background-none">{{ trafficDisplayType == "Bandwidth" ? overview?.traffic.formatted_egress_bytes_per_sec : overview?.traffic.formatted_sent_bytes }}</InputGroupAddon>
                                    </InputGroup>
                                </div>
                                <!-- <div class="text-900 font-medium text-xl">{{ overview?.traffic.bytes_sent }} bytes</div> -->
                            </div>
                            <div class="flex align-items-center justify-content-center bg-purple-100 border-round" style="width:2.5rem;height:2.5rem">
                                <i class="pi pi-upload text-purple-500 text-xl"></i>
                            </div>
                        </div>
                        <div class="flex justify-content-between">
                            <span class="text-500">Upload traffic summary</span>
                            <span class="text-green-500 font-medium">{{ trafficDisplayType }}</span>
                        </div>
                        
                        <!-- <span class="text-green-500 font-medium">{{ overview?.traffic.packet_sent }}</span>
                        <span class="text-500"> packets sent</span> -->
                    </div>
                </div>

                <div class="col-12">
                    <div class="surface-card shadow-2 border-round p-4">
                        <div class="flex justify-content-between align-items-center mb-5">
                            <span class="text-xl text-900 font-medium">Top Remote Addresses</span>
                        </div>
                        <ul class="list-none p-0 m-0">
                            <div v-for=" host in overview?.top_remote_hosts" :key="host.ip_addr">
                                <li class="flex flex-column md:flex-row md:align-items-center md:justify-content-between mb-4">
                                    <div class="flex">
                                        <div>
                                            <span class="block text-900 font-medium mb-1">{{ host.ip_addr }}</span>
                                            <div v-if="host.asn === 0" class="text-600">N/A</div>
                                            <div v-else class="text-600">{{ host.country_code }}, AS{{ host.asn }} {{ host.as_name }}</div>
                                        </div>
                                    </div>
                                    <div class="mt-2 md:mt-0 flex flex-nowrap">
                                        <Button class="p-button-text p-button-plain p-button-rounded mr-1">{{ trafficDisplayType == "Bandwidth" ? host.traffic.formatted_ingress_bytes_per_sec : host.traffic.formatted_received_bytes }} ↓</Button>
                                        <Button class="p-button-text p-button-plain p-button-rounded mr-1">{{ trafficDisplayType == "Bandwidth" ? host.traffic.formatted_egress_bytes_per_sec : host.traffic.formatted_sent_bytes }} ↑</Button>
                                        <!-- <Button class="p-button-text p-button-plain p-button-rounded mr-1" icon="pi pi-info-circle"></Button> -->
                                    </div>
                                </li>
                            </div>
                        </ul>
                    </div>
                </div>
                <div class="col-12 xl:col-6">
                    <div class="surface-card shadow-2 border-round p-4">
                        <div class="text-xl text-900 font-medium mb-4">Top Protocols</div>
                        <ul class="list-none p-0 m-0">
                            <div v-for=" service in overview?.top_app_protocols" :key="generateProtocolPortKey">
                                <li class="flex flex-column md:flex-row md:align-items-center md:justify-content-between mb-4">
                                    <div class="flex">
                                        <div>
                                            <span class="block text-900 font-medium mb-1">{{ service.name }}</span>
                                            <div class="text-600">{{ service.port }}/{{ service.protocol }}</div>
                                        </div>
                                    </div>
                                    <div class="mt-2 md:mt-0 flex flex-nowrap">
                                        <Button class="p-button-text p-button-plain p-button-rounded mr-1">{{ trafficDisplayType == "Bandwidth" ? service.traffic.formatted_ingress_bytes_per_sec : service.traffic.formatted_received_bytes }} ↓</Button>
                                        <Button class="p-button-text p-button-plain p-button-rounded mr-1">{{ trafficDisplayType == "Bandwidth" ? service.traffic.formatted_egress_bytes_per_sec : service.traffic.formatted_sent_bytes }} ↑</Button>
                                        <!-- <Button class="p-button-text p-button-plain p-button-rounded mr-1" icon="pi pi-info-circle"></Button> -->
                                    </div>
                                </li>
                            </div>
                        </ul>
                    </div>
                </div>
                <div class="col-12 xl:col-6">
                    <div class="surface-card shadow-2 border-round p-4">
                        <div class="flex justify-content-between align-items-center mb-5">
                            <span class="text-xl text-900 font-medium">Top Processes</span>
                        </div>
                        <ul class="list-none p-0 m-0">
                            <div v-for=" proc in overview?.top_processes" :key="proc.pid">
                                <li class="flex flex-column md:flex-row md:align-items-center md:justify-content-between mb-4">
                                    <div class="flex">
                                        <div>
                                            <span class="block text-900 font-medium mb-1">{{ proc.name }}</span>
                                            <div class="text-600">{{ proc.pid }}</div>
                                        </div>
                                    </div>
                                    <div class="mt-2 md:mt-0 flex flex-nowrap">
                                        <Button class="p-button-text p-button-plain p-button-rounded mr-1">{{ trafficDisplayType == "Bandwidth" ? proc.traffic.formatted_ingress_bytes_per_sec : proc.traffic.formatted_received_bytes }} ↓</Button>
                                        <Button class="p-button-text p-button-plain p-button-rounded mr-1">{{ trafficDisplayType == "Bandwidth" ? proc.traffic.formatted_egress_bytes_per_sec : proc.traffic.formatted_sent_bytes }} ↑</Button>
                                        <!-- <Button class="p-button-text p-button-plain p-button-rounded mr-1" icon="pi pi-info-circle"></Button> -->
                                    </div>
                                </li>
                            </div>
                        </ul>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <Dialog v-model:visible="interfaceDialogVisible" :modal="false" :closable="true" header="Default Interface" :showHeader="true" :breakpoints="{'960px': '75vw', '640px': '100vw'}" :style="{width: '45vw'}">
        <DataTable :value="interfaceKv"  scrollable scrollHeight="70vh" tableStyle="min-width: 50rem">
                <Column field="key" header="" ></Column>
                <Column field="value" header="" ></Column>
            </DataTable>
        <template #footer>
            <div class="flex border-top-1 pt-5 surface-border justify-content-end align-items-center">
                <Button @click="interfaceDialogVisible = false" icon="pi pi-check" label="OK" class="m-0"></Button>
            </div>
        </template>
    </Dialog>
    <Dialog v-model:visible="ipInfoDialogVisible" :modal="false" :closable="true" header="Public IP Info" :showHeader="true" :breakpoints="{'960px': '75vw', '640px': '100vw'}" :style="{width: '45vw'}">
        <DataTable :value="ipInfoKv"  scrollable scrollHeight="70vh" tableStyle="min-width: 50rem">
                <Column field="key" header="" ></Column>
                <Column field="value" header="" ></Column>
            </DataTable>
        <template #footer>
            <div class="flex border-top-1 pt-5 surface-border justify-content-end align-items-center">
                <Button @click="ipInfoDialogVisible = false" icon="pi pi-check" label="OK" class="m-0"></Button>
            </div>
        </template>
    </Dialog>
</template>
