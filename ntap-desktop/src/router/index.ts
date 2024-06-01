import { createRouter, createWebHistory } from 'vue-router';
import Overview from '../components/Overview.vue';
import Packet from '../components/Packet.vue';
import RemoteAddress from '../components/RemoteAddress.vue';
import Socket from '../components/Socket.vue';
import Process from '../components/Process.vue';

const routes = [
  {
    path: '/',
    name: 'Overview',
    component: Overview,
  },
  {
    path: '/overview',
    name: 'Overview2',
    component: Overview,
  },
  {
    path: '/packet',
    name: 'Packet',
    component: Packet,
  },
  {
    path: '/remote',
    name: 'RemoteAddress',
    component: RemoteAddress,
  },
  {
    path: '/socket',
    name: 'Socket',
    component: Socket,
  },
  {
    path: '/process',
    name: 'Process',
    component: Process,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
