<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { emit, listen } from '@tauri-apps/api/event';
import { WindowUtil } from '../util/window';
import { PacketFrame, PacketDisplayData, PacketFrameExt } from '../types/network';
import { DataTableRowSelectEvent } from 'primevue/datatable';

const packetDataTable = ref();
const maxPacketCount = 10000;
const windowUtil = new WindowUtil();
const tableData = ref<PacketDisplayData[]>([]);
const virtualTableData = ref<PacketDisplayData[]>([]);
const selectedPacket = ref<PacketDisplayData>();
const caputuring = ref(false);
const tableBlocked = ref(false);
const totalRecords = ref(0);
//const defaultDisplaySize = 10;
const maxDisplaySize = 50;
const scrollHeight = ref(0);
//const listData = ref(Array.from({ length: defaultDisplaySize }).map((_, _i) => ``));
const listData = ref<string[]>([]);
const tableLoading = ref(false);
const packetTreeNodes = ref<TreeNode[]>([]);
const dialogVisible = ref(false);
const liveDialogVisible = ref(false);

interface TreeNode {
    key: string;
    label: string;
    data: any;
    icon: string;
    children: TreeNode[];
}

const parsePacketFrame = (packetFrame: PacketFrame): PacketDisplayData => {
    const packetSummary: PacketDisplayData = {
        capture_no: packetFrame.capture_no,
        if_index: packetFrame.if_index,
        if_name: packetFrame.if_name,
        timestamp: packetFrame.timestamp,
        src_addr: "",
        dst_addr: "",
        src_port: null,
        dst_port: null,
        protocol: "",
        packet_len: packetFrame.packet_len,
        info: "",
    };
    if (packetFrame.ip) {
        if (packetFrame.ip.ipv4) {
            packetSummary.src_addr = packetFrame.ip.ipv4.source;
            packetSummary.dst_addr = packetFrame.ip.ipv4.destination;
            packetSummary.protocol = packetFrame.ip.ipv4.next_level_protocol;
        }
        if (packetFrame.ip.ipv6) {
            packetSummary.src_addr = packetFrame.ip.ipv6.source;
            packetSummary.dst_addr = packetFrame.ip.ipv6.destination;
            packetSummary.protocol = packetFrame.ip.ipv6.next_header;
        }
    }
    if (!packetSummary.protocol) {
        if (packetFrame.datalink) {
            if (packetFrame.datalink.ethernet) {
                packetSummary.protocol = packetFrame.datalink.ethernet.ethertype;
            }
        }
    }
    if (packetFrame.transport) {
        if (packetFrame.transport.tcp) {
            packetSummary.src_port = packetFrame.transport.tcp.source;
            packetSummary.dst_port = packetFrame.transport.tcp.destination;
        }
        if (packetFrame.transport.udp) {
            packetSummary.src_port = packetFrame.transport.udp.source;
            packetSummary.dst_port = packetFrame.transport.udp.destination;
        }
    }
    if (!packetSummary.src_addr || !packetSummary.dst_addr) {
        if (packetFrame.datalink) {
            if (packetFrame.datalink.arp) {
                packetSummary.src_addr = packetFrame.datalink.arp.sender_proto_addr;
                packetSummary.dst_addr = packetFrame.datalink.arp.target_proto_addr;
            }else {
                if (packetFrame.datalink.ethernet) {
                    packetSummary.src_addr = packetFrame.datalink.ethernet.source;
                    packetSummary.dst_addr = packetFrame.datalink.ethernet.destination;
                }
            }
        }
    }
    packetSummary.protocol = packetSummary.protocol.toUpperCase();
    return packetSummary;
};

// PacketDisplayData to formatted string.
const packetToString = (packet: PacketDisplayData): string => {
    let packetString = "";
    packetString += packet.timestamp + " [";
    packetString += packet.protocol + "] ";
    if (packet.src_port) {
        packetString += packet.src_addr + ":" + packet.src_port + " > ";
        packetString += packet.dst_addr + ":" + packet.dst_port + " ";
    }else {
        packetString += packet.src_addr + " > ";
        packetString += packet.dst_addr + " ";
    }
    packetString += packet.packet_len + " bytes";
    return packetString;
};

const startPacketCapture = async() => {
    console.log('start packet capture');
    tableBlocked.value = true;
    liveDialogVisible.value = true;
    tableData.value = [];
    virtualTableData.value = [];
    listData.value = [];
    scrollHeight.value = 0;
    const unlisten = await listen<PacketFrame>('packet_frame', (event) => {
        let packet: PacketFrame = event.payload;
        let displayData: PacketDisplayData = parsePacketFrame(packet);
        let packetExt: PacketFrameExt = {
            capture_no: displayData.capture_no,
            if_index: displayData.if_index,
            if_name: displayData.if_name,
            timestamp: displayData.timestamp,
            src_addr: displayData.src_addr,
            dst_addr: displayData.dst_addr,
            src_port: displayData.src_port,
            dst_port: displayData.dst_port,
            protocol: displayData.protocol,
            packet_len: displayData.packet_len,
            info: displayData.info,
            datalink: packet.datalink,
            ip: packet.ip,
            transport: packet.transport,
        };
        tableData.value.push(packetExt);
        listData.value.push(packetToString(displayData));
        if (tableData.value.length > maxPacketCount) {
            tableData.value.shift();
        }
        if (listData.value.length > maxDisplaySize) {
            listData.value.shift();
        }
        scrollHeight.value = listData.value.length * 20;
        totalRecords.value = tableData.value.length;
    });
    console.log('start packet_frame listener');
    invoke('start_packet_capture').then((report) => {
        console.log(report);
        unlisten();
    });
};

const stopPacketCapture = async() => {
    emit('stop_pcap', {
        message: 'stop_pcap',
    });
    tableLoading.value = true;
    virtualTableData.value = tableData.value;
    tableData.value = [];
    liveDialogVisible.value = false;
    tableLoading.value = false;
    tableBlocked.value = false;
    caputuring.value = false;
};

const closeLiveDialog = () => {
    stopPacketCapture();
    listData.value = [];
};

const onChengeCapture = () => {
    if (caputuring.value) {
        startPacketCapture();
        console.log('start capture');
    }else {
        stopPacketCapture();
        console.log('stop capture');
    }
};

const onRowSelect = (event: DataTableRowSelectEvent) => {
    const packet: PacketFrameExt = event.data;
    // Update tree nodes. clear and add new nodes.
    packetTreeNodes.value = [];
    // Frame
    packetTreeNodes.value.push({
        key: '0',
        label: 'Frame',
        data: 'Frame',
        icon: '',
        children: []
    });
    packetTreeNodes.value[0].children.push({
        key: '0-0',
        label: 'Interface' + packet.if_name,
        data: packet.if_index,
        icon: '',
        children: [
            { key: '0-0-0', label: `Interface Index: ${packet.if_index}`, icon: '', data: packet.if_index, children: [] },
            { key: '0-0-1', label: `Interface Name: ${packet.if_name}`, icon: '', data: packet.if_name, children: [] }
        ]
    });
    packetTreeNodes.value[0].children.push({
        key: '0-1',
        label: 'Timestamp: ' + packet.timestamp,
        data: packet.timestamp,
        icon: '',
        children: []
    });
    // Ethernet II
    packetTreeNodes.value.push({
        key: '1',
        label: 'Ethernet II, Src: ' + packet.datalink?.ethernet?.source + ', Dst: ' + packet.datalink?.ethernet?.destination,
        data: 'Ethernet II, Src: ' + packet.datalink?.ethernet?.source + ', Dst: ' + packet.datalink?.ethernet?.destination,
        icon: '',
        children: []
    });
    packetTreeNodes.value[1].children.push({
        key: '1-0',
        label: 'Destination: ' + packet.datalink?.ethernet?.destination,
        data: packet.datalink?.ethernet?.destination ?? '',
        icon: '',
        children: []
    });
    packetTreeNodes.value[1].children.push({
        key: '1-1',
        label: 'Source: ' + packet.datalink?.ethernet?.source,
        data: packet.datalink?.ethernet?.source ?? '',
        icon: '',
        children: []
    });
    // IP
    
    if (packet.ip) {
        if (packet.ip.ipv4) {
            packetTreeNodes.value.push({
                key: '2',
                label: 'Internet Protocol Version 4',
                data: 'Internet Protocol Version 4',
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-0',
                label: 'Version: ' + packet.ip.ipv4.version,
                data: packet.ip.ipv4.version.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-1',
                label: 'Header Length: ' + packet.ip.ipv4.header_length,
                data: packet.ip.ipv4.header_length.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-3',
                label: 'Total Length: ' + packet.ip.ipv4.total_length,
                data: packet.ip.ipv4.total_length.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-4',
                label: 'Identification: ' + packet.ip.ipv4.identification,
                data: packet.ip.ipv4.identification.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-5',
                label: 'Flags: ' + packet.ip.ipv4.flags,
                data: packet.ip.ipv4.flags.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-6',
                label: 'Fragment Offset: ' + packet.ip.ipv4.fragment_offset,
                data: packet.ip.ipv4.fragment_offset.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-7',
                label: 'Time to Live: ' + packet.ip.ipv4.ttl,
                data: packet.ip.ipv4.ttl.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-8',
                label: 'Protocol: ' + packet.ip.ipv4.next_level_protocol,
                data: packet.ip.ipv4.next_level_protocol,
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-9',
                label: 'Header Checksum: ' + packet.ip.ipv4.checksum,
                data: packet.ip.ipv4.checksum.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-10',
                label: 'Source: ' + packet.ip.ipv4.source,
                data: packet.ip.ipv4.source,
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-11',
                label: 'Destination: ' + packet.ip.ipv4.destination,
                data: packet.ip.ipv4.destination,
                icon: '',
                children: []
            });
        }
        if (packet.ip.ipv6) {
            packetTreeNodes.value.push({
                key: '2',
                label: 'Internet Protocol Version 6',
                data: 'Internet Protocol Version 6',
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-0',
                label: 'Version: ' + packet.ip.ipv6.version,
                data: packet.ip.ipv6.version.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-1',
                label: 'Traffic Class: ' + packet.ip.ipv6.traffic_class,
                data: packet.ip.ipv6.traffic_class.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-2',
                label: 'Flow Label: ' + packet.ip.ipv6.flow_label,
                data: packet.ip.ipv6.flow_label.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-3',
                label: 'Payload Length: ' + packet.ip.ipv6.payload_length,
                data: packet.ip.ipv6.payload_length.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-4',
                label: 'Next Header: ' + packet.ip.ipv6.next_header,
                data: packet.ip.ipv6.next_header,
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-5',
                label: 'Hop Limit: ' + packet.ip.ipv6.hop_limit,
                data: packet.ip.ipv6.hop_limit.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-6',
                label: 'Source: ' + packet.ip.ipv6.source,
                data: packet.ip.ipv6.source,
                icon: '',
                children: []
            });
            packetTreeNodes.value[2].children.push({
                key: '2-7',
                label: 'Destination: ' + packet.ip.ipv6.destination,
                data: packet.ip.ipv6.destination,
                icon: '',
                children: []
            });
        }
    }
    // Transport
    if (packet.transport) {
        if (packet.transport.tcp) {
            packetTreeNodes.value.push({
                key: '3',
                label: 'Transmission Control Protocol',
                data: 'Transmission Control Protocol',
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-0',
                label: 'Source Port: ' + packet.transport.tcp.source,
                data: packet.transport.tcp.source.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-1',
                label: 'Destination Port: ' + packet.transport.tcp.destination,
                data: packet.transport.tcp.destination.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-2',
                label: 'Sequence Number: ' + packet.transport.tcp.sequence,
                data: packet.transport.tcp.sequence.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-3',
                label: 'Acknowledgment Number: ' + packet.transport.tcp.acknowledgement,
                data: packet.transport.tcp.acknowledgement.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-5',
                label: 'Flags: ' + packet.transport.tcp.flags,
                data: packet.transport.tcp.flags.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-6',
                label: 'Window Size Value: ' + packet.transport.tcp.window,
                data: packet.transport.tcp.window.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-7',
                label: 'Checksum: ' + packet.transport.tcp.checksum,
                data: packet.transport.tcp.checksum.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-8',
                label: 'Urgent Pointer: ' + packet.transport.tcp.urgent_ptr,
                data: packet.transport.tcp.urgent_ptr.toString(),
                icon: '',
                children: []
            });
        }
        if (packet.transport.udp) {
            packetTreeNodes.value.push({
                key: '3',
                label: 'User Datagram Protocol',
                data: 'User Datagram Protocol',
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-0',
                label: 'Source Port: ' + packet.transport.udp.source,
                data: packet.transport.udp.source.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-1',
                label: 'Destination Port: ' + packet.transport.udp.destination,
                data: packet.transport.udp.destination.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-2',
                label: 'Length: ' + packet.transport.udp.length,
                data: packet.transport.udp.length.toString(),
                icon: '',
                children: []
            });
            packetTreeNodes.value[3].children.push({
                key: '3-3',
                label: 'Checksum: ' + packet.transport.udp.checksum,
                data: packet.transport.udp.checksum.toString(),
                icon: '',
                children: []
            });
        }
    }
    dialogVisible.value = true;
};

const onRowUnselect = (event: DataTableRowSelectEvent) => {
    dialogVisible.value = false;
    console.log(event.data);
}

const onLiveDialogClose = (_event: any) => {
    stopPacketCapture();
    listData.value = [];
};

onMounted(() => {
    windowUtil.mount();
});

onUnmounted(() => {
    windowUtil.unmount();
    stopPacketCapture();
});

</script>

<style scoped>
.p-card, .p-card-title, .p-card-content {
    background-color: var(--surface-ground);
}
</style>

<template>
    <Card class="flex-auto" >
        <template #title>  
            <div class="flex justify-content-between">
                <div class="flex">
                    Packet Capture
                </div>
                <div class="flex">
                    <ToggleButton v-model="caputuring" onLabel="Captureing" offLabel="Stop" onIcon="pi pi-play" offIcon="pi pi-pause" class="mr-2" @change="onChengeCapture" />
                </div>
            </div>
        </template>
        <template #content>
            <BlockUI :blocked="tableBlocked">
                <DataTable ref="packetDataTable" :value="virtualTableData" v-model:selection="selectedPacket" :virtualScrollerOptions="{ itemSize: 20 }" selectionMode="single" dataKey="capture_no" @rowSelect="onRowSelect" @rowUnselect="onRowUnselect" size="small" scrollable :scrollHeight="(windowUtil.windowSize.innerHeight - 200).toString() + 'px'" tableStyle="min-width: 50rem" :loading="tableLoading">
                    <Column field="capture_no" header="No" ></Column>
                    <Column field="timestamp" header="Timestamp" ></Column>
                    <Column field="src_addr" header="SRC Addr" ></Column>
                    <Column field="src_port" header="SRC Port" ></Column>
                    <Column field="dst_addr" header="DST Addr" ></Column>
                    <Column field="dst_port" header="DST Port" ></Column>
                    <Column field="protocol" header="Protocol" ></Column>
                    <Column field="packet_len" header="Length" ></Column>
                    <!-- <Column field="info" header="Info" ></Column> -->
                </DataTable>
            </BlockUI>
        </template>
    </Card>
    <Dialog v-model:visible="dialogVisible" :modal="false" :closable="true" header="Detail" :showHeader="true" :breakpoints="{'960px': '75vw', '640px': '100vw'}" :style="{width: '45vw'}">
        <div class="flex justify-content-between align-items-center w-full">
            <p class="font-medium text-lg text-700 mt-0">No. {{`${selectedPacket?.capture_no}`}}</p>
            <span class="text-500 flex align-items-center"><i class="pi pi-check-square text-lg mr-2"></i>{{`${selectedPacket?.timestamp}`}}</span>
        </div>
        <Tree :value="packetTreeNodes" class="w-full mt-2"></Tree>
        <template #footer>
            <div class="flex border-top-1 pt-5 surface-border justify-content-end align-items-center">
                <Button @click="dialogVisible = false" icon="pi pi-check" label="OK" class="m-0"></Button>
            </div>
        </template>
    </Dialog>
    <Dialog v-model:visible="liveDialogVisible" @update:visible="onLiveDialogClose" maximizable :modal="false" :closable="true" header="Live Capture" :showHeader="true" :breakpoints="{'960px': '75vw', '640px': '100vw'}" :style="{width: '70vw'}">
        <div class="flex justify-content-between align-items-center w-full">
            <p class="font-medium text-lg text-700 mt-0">Live Capture</p>
            <span class="text-500 flex align-items-center"><i class="pi pi-check-square text-lg mr-2"></i>{{ totalRecords }}</span>
        </div>
        <VirtualScroller :items="listData" :itemSize="30" class="border-1 surface-border border-round w-full mt-2" :scroll-height="`${scrollHeight}px`" :style="{width: '600px', height: '50vh'}">
            <template v-slot:item="{ item, options }">
                <div :class="['flex align-items-center p-2', { 'surface-hover': options.odd }]" style="height: 30px">{{ item }}</div>
            </template>
        </VirtualScroller>
        <template #footer>
            <div class="flex border-top-1 pt-5 surface-border justify-content-end align-items-center">
                <Button @click="closeLiveDialog" icon="pi pi-check" label="Stop" class="m-0"></Button>
            </div>
        </template>
    </Dialog>
</template>
